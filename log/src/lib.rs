
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use serde_json::{Result, Value};

fn get_file_path(file_name: &str) -> PathBuf {
    let mut path = if let Some(project_dirs) =
        directories::ProjectDirs::from("rs", "modbus", "GraphicModbus")
    {
        project_dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or(std::path::PathBuf::new())
    };
    path.push(file_name);
    path
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
struct SourceJsonLog {
    start: String,
    finish: String,
    v_dt: Vec<String>,
    v_hash: Vec<String>,
    v_value: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewJsonLog {
    pub start: String,
    pub finish: String,
    pub values: Vec<LogValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogValue {
    pub date_time: String,
    pub hash: String,
    pub value: i32,
}

fn convert_log(source_log: SourceJsonLog) -> NewJsonLog {
    NewJsonLog {
        start: source_log.start,
        finish: source_log.finish,
        values: source_log.v_dt.into_iter()
            .zip(source_log.v_hash.into_iter())
            .zip(source_log.v_value.into_iter())
            .map(|((dt, hash), value)| LogValue{date_time: dt, hash: hash, value: value})
            .collect()
    }
}

pub fn convert_log_file(file_name: &str, from_dir: &str, to_dir: &str) -> std::io::Result<()> {
//     let file_name = "values_14_09_2020__13_24_19_668.json";
    let path = get_file_path(&(from_dir.to_owned() + file_name));
    println!("Path: {:?}", path);
    
    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents);
    let js: SourceJsonLog = serde_json::from_str(&contents)?;
    let js = convert_log(js);
//     dbg!(&js);
    
    let contents = serde_json::to_string_pretty(&js)?;
    let path = get_file_path(&(to_dir.to_owned() + file_name)); // "new_log/"
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref());
    
    Ok(())
}

pub fn open_json_file(file_name: &str) -> NewJsonLog {
    let path = get_file_path(&("new_log".to_owned() + file_name));
    println!("Path: {:?}", path);
    let mut contents = String::new();
    let mut file = File::open(path).expect("Файл не найден");
    file.read_to_string(&mut contents);
    serde_json::from_str(&contents).expect("Error Json Parse")
}
