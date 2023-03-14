use reqwest::header::{HeaderValue, AUTHORIZATION};
use dotenv::dotenv;
use std::env;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use serde_json::value::Value;

mod type_info;

fn main() -> eyre::Result<()>{
    dotenv()?;
    
    let token = env::var("GITHUB_TOKEN")?;

    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;

    let release_url = "https://github.com/rakkyo150/RankedMapData/releases/latest/download/outcome.csv";
    
    let csv_rdr = make_csv_reader(&release_url, &auth_value);

    let mut playlists = get_predicted_values_and_classify_data(csv_rdr)?;

    make_playlist(&mut playlists)?;

    Ok(())
}

fn get_predicted_values_and_classify_data(mut csv_rdr: csv::Reader<reqwest::blocking::Response>) -> Result<type_info::Playlists, eyre::ErrReport> {
    let mut previous_hash = String::new();
    let mut json_result=json!(0);

    let mut playlists = type_info::Playlists::new();


    for record_result in csv_rdr.records() {
        let record: type_info::MapData = match record_result{
            Ok(val) => {
                val.deserialize(None)?
            },
            Err(e)=>{
                println!("couldn't record csv: {}", e);
                continue;
            }
        };

        if previous_hash != record.hash{
            println!("index-{}, hash-{}", record.index, record.hash);
            json_result = get_predicted_values(&record);
        }

        let difficulties = make_difficulties(&record, &json_result);
        add_difficulties_to_playlists(&mut playlists, &record, difficulties);

        previous_hash = record.hash;
    }

    Ok(playlists)
}

fn add_difficulties_to_playlists(playlists: &mut type_info::Playlists, record: &type_info::MapData, difficulties: type_info::Difficulties){
    let (overrated_playlist, underrated_playlist) = playlists.search_playlist(&record.stars).unwrap();

    let targeted_playlist: &mut type_info::Playlist;

    if difficulties.diff > 0.0{
        targeted_playlist = overrated_playlist;
    }
    else{
        targeted_playlist = underrated_playlist;
    }

    match targeted_playlist.search_songs(record.name.as_str(), record.hash.as_str()){
        Some(targeted_songs) => targeted_songs.difficulties.push(difficulties),
        None => {
            let tmp_song = type_info::Songs{
                songName: record.name.to_string(),
                difficulties: vec![difficulties],
                hash: record.hash.to_string()
            };
            targeted_playlist.songs.push(tmp_song);
        }
    }
}

fn make_playlist(playlists: &mut type_info::Playlists) -> Result<(), eyre::ErrReport> {
    for index in 0..15{
        let (overrated_playlist,underrated_playlist) = playlists.search_playlist(&(index as f64)).unwrap();
        let serialized_overrated_playlist = serde_json::to_string_pretty(&overrated_playlist)?;
        let serialized_underrated_playlist = serde_json::to_string_pretty(&underrated_playlist)?;
        let mut file = File::create(format!("./overrated_playlist_{}.json", index))?;
        file.write_all(serialized_overrated_playlist.as_bytes())?;
        let mut file = File::create(format!("./underrated_playlist_{}.json", index))?;
        file.write_all(serialized_underrated_playlist.as_bytes())?;
    }
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


fn make_difficulties(record: &type_info::MapData, json_result: &Value) -> type_info::Difficulties {
    let predicted_values = match record.difficulty.as_str() {
        "Easy" => json_result["Standard-Easy"].as_f64().unwrap(),
        "Normal" => json_result["Standard-Normal"].as_f64().unwrap(),
        "Hard" => json_result["Standard-Hard"].as_f64().unwrap(),
        "Expert" => json_result["Standard-Expert"].as_f64().unwrap(),
        "ExpertPlus" => json_result["Standard-ExpertPlus"].as_f64().unwrap(),
        &_ => 0.0
    };

    let difficulties = type_info::Difficulties{
        name: record.difficulty.to_string(),
        characteristic: record.characteristic.to_string(),
        diff: record.stars - predicted_values
    };

    difficulties
}

fn get_predicted_values(record: &type_info::MapData) -> Value {
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