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

fn main() {
    let result = get_ranked_map_data();
    match  result {
        Ok(())=> println!("Finish!"),
        Err(err) => println!("Error: {}",err)
    };
}

fn get_ranked_map_data() -> eyre::Result<()> {
    dotenv()?;

    let token = env::var("GITHUB_TOKEN")?;

    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;

    let release_url = "https://github.com/rakkyo150/RankedMapData/releases/latest/download/outcome.csv";
    
    let csv_rdr = make_csv_reader(&release_url, &auth_value);

    let (overrated_songs, underrated_songs) = get_predicted_values_and_classify_data(csv_rdr)?;

    make_playlist(overrated_songs, underrated_songs)?;

    Ok(())
}

fn get_predicted_values_and_classify_data(mut csv_rdr: csv::Reader<reqwest::blocking::Response>) -> Result<(Vec<type_info::Songs>, Vec<type_info::Songs>), eyre::ErrReport> {
    let mut overrated_songs: Vec<type_info::Songs> = Vec::new();
    let mut underrated_songs: Vec<type_info::Songs> = Vec::new();
    let mut overrated_difficulties: Vec<type_info::Difficulties> = Vec::new();
    let mut underrated_difficulties: Vec<type_info::Difficulties> = Vec::new();
    let mut previous_hash = String::new();
    let mut json_result: Value = json!(0);
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

        (previous_hash, overrated_difficulties, overrated_songs, underrated_difficulties, underrated_songs, json_result) = update_and_classify_data(&previous_hash ,&record, overrated_difficulties, overrated_songs, underrated_difficulties, underrated_songs, json_result);
    }
    Ok((overrated_songs, underrated_songs))
}

fn make_playlist(overrated_songs: Vec<type_info::Songs>, underrated_songs: Vec<type_info::Songs>) -> Result<(), eyre::ErrReport> {
    let overrated_playlist = type_info::Playlist{
        playlistTitle: "Overrated Playlist".to_string(),
        songs : overrated_songs
    };
    let underrated_playlist = type_info::Playlist{
        playlistTitle: "Underrated Playlist".to_string(),
        songs: underrated_songs
    };
    let serialized_overrated_playlist = serde_json::to_string_pretty(&overrated_playlist)?;
    let serialized_underrated_playlist = serde_json::to_string_pretty(&underrated_playlist)?;
    let mut file = File::create("./overrated.json")?;
    file.write_all(serialized_overrated_playlist.as_bytes())?;
    let mut file = File::create("./underrated.json")?;
    file.write_all(serialized_underrated_playlist.as_bytes())?;
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

fn update_and_classify_data (previous_hash: &String,record: &type_info::MapData, mut overrated_difficulties: Vec<type_info::Difficulties>, mut overrated_songs: Vec<type_info::Songs>, mut underrated_difficulties: Vec<type_info::Difficulties>, mut underrated_songs: Vec<type_info::Songs>, mut json_result: Value) -> (String, Vec<type_info::Difficulties>, Vec<type_info::Songs>, Vec<type_info::Difficulties>, Vec<type_info::Songs>, Value) {    
    println!("index : {}",record.index);

    if *previous_hash != record.hash {
        thread::sleep(Duration::from_secs(1));
        json_result = get_predicted_values(record);
    }

    let tmp_difficulties = make_tmp_difficulties(record, &json_result);

    if tmp_difficulties.diff > 0.0{
        println!("overrated: hash-{} , diff-{}", record.hash ,tmp_difficulties.diff);
        if *previous_hash != record.hash{
            add_and_clear_all_difficulties(&mut overrated_difficulties, record, previous_hash, &mut overrated_songs, &mut underrated_difficulties, &mut underrated_songs);
        }
        overrated_difficulties.push(tmp_difficulties);
    }
    else{
        println!("underrated: hash-{} , diff-{}", record.hash ,tmp_difficulties.diff);
        if *previous_hash != record.hash{
            add_and_clear_all_difficulties(&mut overrated_difficulties, record, previous_hash, &mut overrated_songs, &mut underrated_difficulties, &mut underrated_songs);
        }
        underrated_difficulties.push(tmp_difficulties);
    }

    (record.hash.to_string(), overrated_difficulties, overrated_songs, underrated_difficulties, underrated_songs, json_result)
}

fn make_tmp_difficulties(record: &type_info::MapData, json_result: &Value) -> type_info::Difficulties {
    let mut predicted_values: f64 = 0.0;

    if record.difficulty == "Easy" {
        predicted_values = json_result["Standard-Easy"].as_f64().unwrap();
    }
    else if record.difficulty == "Normal" {
        predicted_values = json_result["Standard-Normal"].as_f64().unwrap();
    }
    else if record.difficulty == "Hard" {
        predicted_values = json_result["Standard-Hard"].as_f64().unwrap();
    }
    else if record.difficulty == "Expert" {
        predicted_values = json_result["Standard-Expert"].as_f64().unwrap();
    }
    else if record.difficulty == "ExpertPlus" {
        predicted_values = json_result["Standard-ExpertPlus"].as_f64().unwrap();
    }

    let tmp_difficulties = type_info::Difficulties{
        name: record.difficulty.to_string(),
        characteristic: record.characteristic.to_string(),
        diff: record.stars - predicted_values
    };
    tmp_difficulties
}

fn add_and_clear_all_difficulties(overrated_difficulties: &mut Vec<type_info::Difficulties>, record: &type_info::MapData, previous_hash: &String, overrated_songs: &mut Vec<type_info::Songs>, underrated_difficulties: &mut Vec<type_info::Difficulties>, underrated_songs: &mut Vec<type_info::Songs>) {
    if overrated_difficulties.len() != 0{
        add_and_clear_difficulties(record, overrated_difficulties, previous_hash, overrated_songs);
    }
    
    if underrated_difficulties.len() != 0{
        add_and_clear_difficulties(record, underrated_difficulties, previous_hash, underrated_songs);
    }
}

fn add_and_clear_difficulties(record: &type_info::MapData, difficulties: &mut Vec<type_info::Difficulties>, previous_hash: &String, songs: &mut Vec<type_info::Songs>) {
    let tmp_songs = type_info::Songs{
        songName: record.name.to_string(),
        difficulties: difficulties.to_vec(),
        hash: previous_hash.to_string()
    };
    songs.push(tmp_songs);
    difficulties.clear();
}

fn get_predicted_values(record: &type_info::MapData) -> Value {
    let url = format!("https://predictstarnumber.onrender.com/api2/hash/{}", record.hash);

    // サーバーの再起動が結構長いので
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(300)).build().unwrap();
        
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