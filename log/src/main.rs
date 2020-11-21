
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use serde_json::{Result, Value};

fn main() -> std::io::Result<()> {
    convert_log_file("values_14_09_2020__13_24_19_668.json", "Log/", "new_log")
}

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
struct NewJsonLog {
    start: String,
    finish: String,
    values: Vec<LogValue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogValue {
    date_time: String,
    hash: String,
    value: i32,
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

fn convert_log_file(file_name: &str, from_dir: &str, to_dir: &str) -> std::io::Result<()> {
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

// За 11 секунд и 30-40 мб озу
fn test_speed() -> std::io::Result<()> {
    let paths = vec![
        "values_27_08_2020__13_08_30_042.json",
        "values_07_09_2020__13_02_37_096.json",
        "values_25_08_2020__13_41_06_111.json",
        "values_26_08_2020__16_26_04_840.json",
        "values_07_09_2020__16_13_35_221.json",
        "values_28_08_2020__16_57_26_959.json",
        "values_08_09_2020__14_28_27_576.json",
        "values_08_09_2020__14_28_33_906.json",
        "values_10_09_2020__15_36_13_274.json",
        "values_28_08_2020__17_06_20_523.json",
        "values_21_08_2020__17_31_00_188.json",
        "values_26_08_2020__15_48_12_214.json",
        "values_26_08_2020__16_05_51_804.json",
        "values_25_08_2020__15_15_21_933.json",
        "values_24_08_2020__19_19_10_684.json",
        "values_24_08_2020__19_03_16_045.json",
        "values_24_08_2020__18_31_00_766.json",
    ];
    for _ in 1..10 {
        for path in &paths {
            convert_log_file(path, "Log/", "test_log")?;
        }
    }
    Ok(())
}
