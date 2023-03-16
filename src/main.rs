use reqwest::header::{HeaderValue, AUTHORIZATION};
use dotenv::dotenv;
use map_and_playlist::Difficulties;
use std::env;
use serde_json::json;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::thread;
use std::time::Duration;
use serde_json::value::Value;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use std::result::Result;

mod map_and_playlist;

fn main() -> eyre::Result<()>{
    dotenv()?;
    
    let token = env::var("GITHUB_TOKEN")?;

    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;

    let release_url = "https://github.com/rakkyo150/RankedMapData/releases/latest/download/outcome.csv";
    
    let csv_rdr = make_csv_reader(&release_url, &auth_value);

    let mut playlists = get_predicted_values_and_classify_data(csv_rdr)?;

    make_sorted_playlists(&mut playlists)?;

    Ok(())
}

fn get_predicted_values_and_classify_data(mut csv_rdr: csv::Reader<reqwest::blocking::Response>) -> Result<map_and_playlist::Playlists, eyre::ErrReport> {
    let mut previous_hash = String::new();
    let mut json_result=json!(0);

    let mut playlists = map_and_playlist::Playlists::new();

    // csv_rdrから直接for文回すと、なぜかrequest or response body error: error reading a body from connection: end of file before message length reachedエラーに偶に遭遇することがあるので、一旦全データを確保しておく
    let mut record_container: Vec<map_and_playlist::MapData> = vec![];
    for record_result in csv_rdr.records() {
        match record_result{
            Ok(val) => {
                record_container.push(val.deserialize(None)?)
            },
            Err(e)=>{
                println!("Failed to deserialize csv: {}", e);
                continue;
            }
        };
    }

    for record in record_container
    {
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

        // break;
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

fn make_sorted_playlists(playlists: &mut map_and_playlist::Playlists) -> Result<(), eyre::ErrReport> {
    let mut file_paths: Vec<String> = vec![];

    for index in 0..15{
        let (overrated_playlist,underrated_playlist) = playlists.search_playlist_set(&(index as f64)).unwrap();
        let a_little_overrated_playlist_name = format!("./a_little_overrated_playlist_{}.json", index);
        let fairly_overrated_playlist_name = format!("./fairly_overrated_playlist_{}.json", index);
        let very_overrated_playlist_name = format!("./very_overrated_playlist_{}.json", index);
        let a_little_underrated_playlist_name = format!("./a_little_underrated_playlist_{}.json", index);
        let fairly_underrated_playlist_name = format!("./fairly_underrated_playlist_{}.json", index);
        let very_underrated_playlist_name = format!("./very_underrated_playlist_{}.json", index);

        overrated_playlist.a_little_version.sort();
        overrated_playlist.fairly_version.sort();
        overrated_playlist.very_version.sort();
        underrated_playlist.a_little_version.sort();
        underrated_playlist.fairly_version.sort();
        underrated_playlist.very_version.sort();
        
        make_playlist(&overrated_playlist.a_little_version, &a_little_overrated_playlist_name)?;
        make_playlist(&overrated_playlist.fairly_version, &fairly_overrated_playlist_name)?;
        make_playlist(&overrated_playlist.very_version, &very_overrated_playlist_name)?;
        make_playlist(&underrated_playlist.a_little_version, &a_little_underrated_playlist_name)?;
        make_playlist(&underrated_playlist.fairly_version, &fairly_underrated_playlist_name)?;
        make_playlist(&underrated_playlist.very_version, &very_underrated_playlist_name)?;

        file_paths.extend(vec![a_little_overrated_playlist_name, fairly_overrated_playlist_name, very_overrated_playlist_name, a_little_underrated_playlist_name, fairly_underrated_playlist_name, very_underrated_playlist_name]);
    }

    let zip_file_path = "./all.zip";

    if let Err(error) = create_zip(&file_paths, zip_file_path) {
        println!("Error creating the zip file: {:?}", error);
    } else {
        println!("The zip file created successfully.")
    }
    Ok(())
}

fn make_playlist(playlist: &map_and_playlist::Playlist, playlist_name: &String) -> Result<(), eyre::ErrReport> {
    let serialized_playlist = serde_json::to_string_pretty(playlist)?;
    let mut file = File::create(playlist_name)?;
    file.write_all(serialized_playlist.as_bytes())?;
    Ok(())
}

fn create_zip(file_paths: &[String], zip_file_path: &str) -> Result<(), Error> {
    let zip_path = Path::new(zip_file_path);
    let file = match File::create(&zip_path) {
        Ok(file) => file,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Could not create the zip file: {}", e))),
    };
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

    for file_path in file_paths {
        let single_file_path = Path::new(file_path);
        let mut file = match File::open(&single_file_path) {
            Ok(file) => file,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Unable to open {}. Reason: {}", file_path, e)))
        };
        let mut contents = vec![];
        let _ = file.read_to_end(&mut contents);

        zip.start_file(single_file_path.to_string_lossy()[2..].to_string(), options)?;
        let _ = zip.write_all(&contents);
    }

    let _ = zip.finish();

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