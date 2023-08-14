use reqwest::header::{HeaderValue, AUTHORIZATION};
use dotenv::dotenv;
use map_and_playlist::Difficulties;
use tract_onnx::onnx;
use tract_onnx::prelude::{Framework, InferenceFact, tvec, InferenceModelExt, Tensor};
use tract_onnx::tract_hir::tract_ndarray::Array2;
use std::collections::HashMap;
use std::env;
use serde_json::json;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write, BufReader};
use serde_json::value::Value;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use std::result::Result;
use tract_onnx::prelude::Datum;

use crate::map_and_playlist::MapData;

mod map_and_playlist;

fn main() -> eyre::Result<()>{
    let path = Path::new("./.env");
    if path.exists() {
        dotenv()?;
    }
    
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

    let mut model_buf: Vec<u8> = Vec::new();
    let mut dictionary = HashMap::new();
    
    for (index , record) in record_container.iter().enumerate()
    {
        if index == record_container.len() - 1 {
            dictionary_insert(&record, &mut dictionary, &mut model_buf);
            let difficulties = make_difficulties(&record, json!(dictionary));
            match difficulties {
                Ok(value) => add_difficulties_to_playlists(&mut playlists, &record, value),
                Err(e) => println!("Failed to make difficulties: {}", e)
            }
        } else if previous_hash == record.hash || previous_hash.is_empty() {
            dictionary_insert(&record, &mut dictionary, &mut model_buf);
        } else {
            let difficulties = make_difficulties(&record, json!(dictionary));
            match difficulties {
                Ok(value) => add_difficulties_to_playlists(&mut playlists, &record, value),
                Err(e) => println!("Failed to make difficulties: {}", e)
            }
            dictionary_insert(&record, &mut dictionary, &mut model_buf);
        }
        
        previous_hash = record.hash.to_owned();
        
        fn dictionary_insert(record: &MapData, dictionary: &mut HashMap<String, f64>, model_buf: &mut Vec<u8>) {
            println!("index-{}, hash-{}, difficulty-{}", record.index, record.hash, record.difficulty);
            let predicted_value = get_predicted_values(&record, model_buf);
            dictionary.insert(format!("{}-{}", record.characteristic, record.difficulty), predicted_value);
        }

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


fn make_difficulties(record: &map_and_playlist::MapData, json_result: Value) -> Result<Difficulties, String> {
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

fn get_predicted_values(record: &map_and_playlist::MapData, model_buf: &mut Vec<u8> ) -> f64 {
    if model_buf.len() == 0 {
        println!("load model");
        *model_buf = load_model().unwrap();
    }
    let model = onnx().model_for_read(&mut BufReader::new(&model_buf[..]))
        .unwrap()
        .with_input_fact(0, InferenceFact::dt_shape(f64::datum_type(), tvec![1, 15]))
        .unwrap()
        .with_output_fact(0, InferenceFact::dt_shape(f64::datum_type(), tvec![1, 1]))
        .unwrap()
        .into_optimized()
        .unwrap()
        .into_runnable()
        .unwrap();

    let difficulties = match record.difficulty.as_str() {
        "Easy" => 0.0,
        "Normal" => 1.0,
        "Hard" => 2.0,
        "Expert" => 3.0,
        "ExpertPlus" => 4.0,
        _ => 0.0
    };
    let sage_score = record.sageScore.parse().unwrap_or_else(|_err| {
        0.0
    });
    let chroma = if record.chroma == "True" {
        1.0
    } else {
        0.0
    };

    // Create an input Tensor
    let data: Vec<f64> = vec![record.bpm, record.duration, difficulties, sage_score ,record.njs , record.offset ,record.notes as f64, record.bombs as f64, record.obstacles as f64, record.nps, record.events, chroma, record.errors as f64, record.warns as f64, record.resets as f64];
    let shape = [1, 15];
    let input = Tensor::from(Array2::<f64>::from_shape_vec(shape, data).unwrap());

    // Run the model
    let outputs = model.run(tvec!(input.into())).unwrap();
    
    // Extract the output tensor
    let output_tensor = &outputs[0];
    
    // Extract the result values
    let result = output_tensor.to_array_view::<f64>().unwrap();
    println!("result: {:?}", result);
    let predicted_value = result[[0, 0]];
    println!("predicted_value: {}", predicted_value);

    predicted_value
}


fn load_model() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let token = env::var("GITHUB_TOKEN")?;
    let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;
    let model_asset_endpoint = "https://github.com/rakkyo150/PredictStarNumberHelper/releases/latest/download/model.onnx";
    let client = reqwest::blocking::Client::new();
    let mut model_asset_response= match client.get(model_asset_endpoint).header(AUTHORIZATION, auth_value).send() {
        Ok(response) => response,
        Err(e) => panic!("Error: {}", e),
    };
    let mut buf = Vec::new();
    model_asset_response.read_to_end(&mut buf)?;

    /*
    let model_file_path = Path::new("model.pickle");
    let mut model_file = File::create(model_file_path)?;
    model_file.write_all(&buf)?;
    */

    Ok(buf)
}