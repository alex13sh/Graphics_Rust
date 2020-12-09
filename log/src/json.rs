use std::fs::File;
use std::io::prelude::*;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use super::LogValue;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SourceJsonLog {
    start: Option<String>,
    finish: Option<String>,
    v_dt: Vec<String>,
    v_hash: Vec<String>,
    v_value: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewJsonLog {
    pub start: String,
    pub finish: String,
    pub values: Vec<LogValue>,
}

use std::collections::HashSet;
impl NewJsonLog {
    pub fn get_all_hash(&self) -> HashSet<String> {
        self.values.iter().fold(HashSet::new(), |mut hashs, val| {
            hashs.insert(val.hash.clone());
            hashs
        })
    }
}


pub fn open_json_file(file_name: &str) -> NewJsonLog {
    let path = super::get_file_path(&("new_log/".to_owned() + file_name));
    println!("Path: {:?}", path);
    let mut contents = String::new();
    let mut file = File::open(path).expect("Файл не найден");
    file.read_to_string(&mut contents);
    serde_json::from_str(&contents).expect("Error Json Parse")
}


#[cfg(feature = "convert")]
pub mod convert {
    use super::*;
    use crate as log;
    
//     type MyResult = Result<(), Box<dyn std::error::Error>>;
    
    fn convert_log(source_log: SourceJsonLog) -> NewJsonLog {
        NewJsonLog {
            start: source_log.start.unwrap_or("".into()),
            finish: source_log.finish.unwrap_or("".into()),
            values: source_log.v_dt.into_iter()
                .zip(source_log.v_hash.into_iter())
                .zip(source_log.v_value.into_iter())
                .map(|((dt, hash), value)| log::LogValue{
                    date_time: crate::NaiveDateTime::parse_from_str(&dt, "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
                    hash: hash, 
                    value: f32::from_bits(value as u32)}
                )
                .collect()
        }
    }

    pub fn convert_log_file(file_name: &str, from_dir: &str, to_dir: &str) -> crate::MyResult {
    //     let file_name = "values_14_09_2020__13_24_19_668.json";
        let path = log::get_file_path(&(from_dir.to_owned() + file_name));
        println!("Path: {:?}", path);
        
        let mut contents = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut contents);
        let js: SourceJsonLog = serde_json::from_str(&contents)?;
        let js = convert_log(js);
    //     dbg!(&js);
        
        let contents = serde_json::to_string_pretty(&js)?;
        let path = log::get_file_path(&(to_dir.to_owned() + file_name)); // "new_log/"
        let mut file = File::create(path)?;
        file.write_all(contents.as_ref());
        
        Ok(())
    }
}
