use std::fs::File;
use std::path::PathBuf;
use std::error::Error;
 
// type DateTime = chrono::DateTime<chrono::Local>;
// type DateTimeFix = chrono::DateTime<chrono::FixedOffset>; 
// use chrono::Duration;

use crate::{naive_date_time_from_str, naive_date_time_to_str};

use serde::{Deserialize, Serialize};
 #[derive(Debug, Serialize, Deserialize)]
pub struct SessionTime {
    #[serde(serialize_with = "naive_date_time_to_str")]
    #[serde(deserialize_with = "naive_date_time_from_str")]
    pub start: crate::NaiveDateTime,
    #[serde(serialize_with = "naive_date_time_to_str")]
    #[serde(deserialize_with = "naive_date_time_from_str")]
    pub finish: crate::NaiveDateTime,
    pub fileName: Option<String>,
    
    #[serde(skip)]
    pub values: Option<Vec<crate::LogValue>>,
}
 
impl SessionTime {
    pub fn set_fileName(&mut self, fileName: String) {
        self.fileName = Some(fileName);
    }
}

pub fn test_read_csv_1(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file_path = super::get_file_path(file_path);
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    for result in rdr.deserialize() {
        let record:SessionTime = result?;
        println!("{:?}", record);
    }
    Ok(())
}
 
pub fn write_values(fileName: &PathBuf, values: Vec<crate::LogValue>) -> crate::MyResult {
    let file = File::create(fileName)?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
    for value in values {
        wrt.serialize(value)?;
    }
    
    Ok(())
}

pub fn read_values(fileName: &PathBuf) -> Option<Vec<crate::LogValue>> {
    let file = File::open(fileName).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    Some(rdr.deserialize()
        .filter_map(|res| res.ok())
        .collect()
    )
}

pub fn write_session(fileName: &PathBuf, session: Vec<SessionTime>) -> crate::MyResult {
    let file = File::create(fileName)?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
    for s in session {
        wrt.serialize(s)?;
    }
    
    Ok(())
}

pub fn read_session(fileName: &PathBuf) -> Option<Vec<SessionTime>> {
    let file = File::open(fileName).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    Some(rdr.deserialize()
        .filter_map(|res| res.ok())
        .collect()
    )
}

pub fn read_session_full(fileName: &PathBuf) -> Option<Vec<SessionTime>> {
    let mut sessions = read_session(&fileName)?;
    for s in &mut sessions {
        if let Some(ref s_fileName) = s.fileName {
            s.values = read_values(&fileName.with_file_name(s_fileName));
        }
    }
    Some(sessions)
}
