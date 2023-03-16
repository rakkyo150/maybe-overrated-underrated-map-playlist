use reqwest::header::{HeaderValue, AUTHORIZATION};
use dotenv::dotenv;
use map_and_playlist::Difficulties;
use std::env;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use serde_json::value::Value;

mod map_and_playlist;

fn main() -> eyre::Result<()>{
    dotenv()?;
    
    let token = env::var("GITHUB_TOKEN")?;

    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;

    let release_url = "https://github.com/rakkyo150/RankedMapData/releases/latest/download/outcome.csv";
    
    let csv_rdr = make_csv_reader(&release_url, &auth_value);

    let mut playlists = get_predicted_values_and_classify_data(csv_rdr)?;

    make_playlists(&mut playlists)?;

    Ok(())
}

fn get_predicted_values_and_classify_data(mut csv_rdr: csv::Reader<reqwest::blocking::Response>) -> Result<map_and_playlist::Playlists, eyre::ErrReport> {
    let mut previous_hash = String::new();
    let mut json_result=json!(0);

    let mut playlists = map_and_playlist::Playlists::new();


    for record_result in csv_rdr.records() {
        let record: map_and_playlist::MapData = match record_result{
            Ok(val) => {
                val.deserialize(None)?
            },
            Err(e)=>{
                println!("Failed to deserialize csv: {}", e);
                continue;
            }
        };

        if previous_hash != record.hash{
            println!("index-{}, hash-{}", record.index, record.hash);
            json_result = get_predicted_values(&record);
        }

        let difficulties = make_difficulties(&record, &json_result);
        match difficulties {
            Ok(value) => add_difficulties_to_playlists(&mut playlists, &record, value),
            Err(e) => println!("Failed to make difficulties: {}", e)
        }

        previous_hash = record.hash;
    }

    Ok(playlists)
}

fn add_difficulties_to_playlists(playlists: &mut map_and_playlist::Playlists, record: &map_and_playlist::MapData, difficulties: map_and_playlist::Difficulties){
    let (overrated_playlist, underrated_playlist) = playlists.search_playlist_set(&record.stars).unwrap();

    let targeted_playlist: &mut map_and_playlist::Playlist;

    if 0.0 <= difficulties.diff && difficulties.diff < 0.5{
        targeted_playlist = &mut overrated_playlist.a_little_version;
    }
    else if 0.5 <= difficulties.diff && difficulties.diff < 1.0{
        targeted_playlist = &mut overrated_playlist.fairly_version;
    }
    else if 1.0 <= difficulties.diff{
        targeted_playlist = &mut  overrated_playlist.very_version;
    }
    else if -0.5 < difficulties.diff && difficulties.diff < 0.0{
        targeted_playlist = &mut underrated_playlist.a_little_version;
    }
    else if -1.0 < difficulties.diff && difficulties.diff <= -0.5{
        targeted_playlist = &mut underrated_playlist.fairly_version;
    }
    else{
        targeted_playlist = &mut underrated_playlist.very_version;
    }

    match targeted_playlist.search_songs(record.name.as_str(), record.hash.as_str()){
        Some(targeted_songs) => targeted_songs.difficulties.push(difficulties),
        None => {
            let tmp_song = map_and_playlist::Songs{
                songName: record.name.to_string(),
                difficulties: vec![difficulties],
                hash: record.hash.to_string()
            };
            targeted_playlist.songs.push(tmp_song);
        }
    }
}

fn make_playlists(playlists: &mut map_and_playlist::Playlists) -> Result<(), eyre::ErrReport> {
    for index in 0..15{
        let (overrated_playlist,underrated_playlist) = playlists.search_playlist_set(&(index as f64)).unwrap();
        let a_little_overrated_playlist_name = format!("./a_little_overrated_playlist_{}.json", index);
        let fairly_overrated_playlist_name = format!("./fairly_overrated_playlist_{}.json", index);
        let very_overrated_playlist_name = format!("./very_overrated_playlist_{}.json", index);
        let a_little_underrated_playlist_name = format!("./a_little_underrated_playlist_{}.json", index);
        let fairly_underrated_playlist_name = format!("./fairly_underrated_playlist_{}.json", index);
        let very_underrated_playlist_name = format!("./very_underrated_playlist_{}.json", index);

        make_playlist(&overrated_playlist.a_little_version, a_little_overrated_playlist_name)?;
        make_playlist(&overrated_playlist.fairly_version, fairly_overrated_playlist_name)?;
        make_playlist(&overrated_playlist.very_version, very_overrated_playlist_name)?;
        make_playlist(&underrated_playlist.a_little_version, a_little_underrated_playlist_name)?;
        make_playlist(&underrated_playlist.fairly_version, fairly_underrated_playlist_name)?;
        make_playlist(&underrated_playlist.very_version, very_underrated_playlist_name)?;
    }
    Ok(())
}

fn make_playlist(playlist: &map_and_playlist::Playlist, playlist_name: String) -> Result<(), eyre::ErrReport> {
    let serialized_playlist = serde_json::to_string_pretty(playlist)?;
    let mut file = File::create(playlist_name)?;
    file.write_all(serialized_playlist.as_bytes())?;
    Ok(())
}

fn make_csv_reader(release_url: &str, auth_value: &HeaderValue) -> csv::Reader<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    let response_csv = match client.get(release_url).header(AUTHORIZATION, auth_value).send() {
        Ok(response) => response,
        Err(e) => panic!("Error: {}", e),
    };

    let csv_rdr = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .delimiter(b',')
                            .from_reader(response_csv);
    csv_rdr
}


fn make_difficulties(record: &map_and_playlist::MapData, json_result: &Value) -> Result<Difficulties, String> {
    let predicted_values = match record.difficulty.as_str() {
        "Easy" => json_result["Standard-Easy"].as_f64(),
        "Normal" => json_result["Standard-Normal"].as_f64(),
        "Hard" => json_result["Standard-Hard"].as_f64(),
        "Expert" => json_result["Standard-Expert"].as_f64(),
        "ExpertPlus" => json_result["Standard-ExpertPlus"].as_f64(),
        &_ => return Err("Error record difficulty".to_string())
    };

    match predicted_values {
        Some(value) => {
            let difficulties = map_and_playlist::Difficulties{
                name: record.difficulty.to_string(),
                characteristic: record.characteristic.to_string(),
                diff: record.stars - value
            };
            return Ok(difficulties)
        },
        None => return Err(format!("There is something wrong with {}", json_result))
    };
}

fn get_predicted_values(record: &map_and_playlist::MapData) -> Value {
    let url = format!("https://predictstarnumber.onrender.com/api2/hash/{}", record.hash);

    // サーバーの再起動が結構長いので
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(300)).build().unwrap();
        
    thread::sleep(Duration::from_secs(1));
    let response = match client.get(url).send() {
        Ok(response) => response,
        Err(e) => panic!("Error: {}", e),
    };

    let json_result = response.json::<serde_json::Value>().unwrap_or_else(|err| {
        eprintln!("Failed to deserialize json: {:?}", err.to_string());
        std::process::exit(1);
    });

    json_result
}