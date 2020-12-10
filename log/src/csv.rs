use std::fs::File;
use std::path::PathBuf;
use std::error::Error;
 
// type DateTime = chrono::DateTime<chrono::Local>;
// type DateTimeFix = chrono::DateTime<chrono::FixedOffset>; 
// use chrono::Duration;

use crate::{date_time_from_str, date_time_to_str};

use serde::{Deserialize, Serialize};
 #[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTime {
    #[serde(serialize_with = "date_time_to_str")]
    #[serde(deserialize_with = "date_time_from_str")]
    pub start: crate::DateTime,
    #[serde(serialize_with = "date_time_to_str")]
    #[serde(deserialize_with = "date_time_from_str")]
    pub finish: crate::DateTime,
    pub file_name: Option<String>,
    
    #[serde(skip)]
    pub values: Option<Vec<crate::LogValue>>,
}
 
impl SessionTime {
    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Some(file_name);
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
 
pub fn write_values(file_name: &PathBuf, values: Vec<crate::LogValue>) -> crate::MyResult {
    let file = File::create(file_name)?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
    for value in values {
        wrt.serialize(value)?;
    }
    
    Ok(())
}

pub fn read_values(file_name: &PathBuf) -> Option<Vec<crate::LogValue>> {
    let file = File::open(file_name).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    Some(rdr.deserialize()
        .filter_map(|res| res.ok())
        .collect()
    )
}

pub fn write_session(file_name: &PathBuf, session: Vec<SessionTime>) -> crate::MyResult {
    let file = File::create(file_name)?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
    for s in session {
        wrt.serialize(s)?;
    }
    
    Ok(())
}

pub fn read_session(file_name: &PathBuf) -> Option<Vec<SessionTime>> {
    let file = File::open(file_name).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    Some(rdr.deserialize()
        .filter_map(|res| res.ok())
        .collect()
    )
}

pub fn read_session_full(file_name: &PathBuf) -> Option<Vec<SessionTime>> {
    let mut sessions = read_session(&file_name)?;
    for s in &mut sessions {
        if let Some(ref s_file_name) = s.file_name {
            s.values = read_values(&file_name.with_file_name(s_file_name));
        }
    }
    Some(sessions)
}
