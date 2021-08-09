#![allow(dead_code, unused_variables, unused_imports)]

pub use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
pub use chrono::{SecondsFormat, Offset, FixedOffset, Duration};

type DateTimeLocal = chrono::DateTime<chrono::Local>;
pub type DateTimeFix = chrono::DateTime<chrono::FixedOffset>;
// type DateTimeMSK = chrono::DateTime<MSK>;
pub type DateTime = DateTimeFix;

pub fn date_time_now() -> DateTime {
    DateTime::from(chrono::Local::now())
//         .east(3*60*60)
}

pub fn date_time_to_string_name(dt: &DateTime) -> String {
    dt.format("%d_%m_%Y__%H_%M_%S_%.f")
        .to_string().replace("_.", "_")
}

pub fn date_time_to_string_name_short(dt: &DateTime) -> String {
    (*dt+Duration::hours(3)).format("%d_%m_%Y__%H_%M_%S")
        .to_string()
}

pub mod json;
pub mod csv;

#[cfg(feature = "convert")]
pub mod convert;

// pub use json::*;
// pub use csv::*;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;

pub fn get_file_path(file_name: &str) -> PathBuf {
    let mut path = if let Some(project_dirs) =
        directories::ProjectDirs::from("rs", "modbus", "GraphicModbus")
    {
        project_dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or(PathBuf::new())
    };
    path.push(file_name);
    path
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogValue {
    #[serde(deserialize_with = "date_time_from_str")]
    #[serde(serialize_with = "date_time_to_str")]
    pub date_time: DateTimeFix,
    pub hash: String,
    pub value: f32,
}

impl LogValue {
    pub fn new(hash: String, value: f32) -> Self {
//         dbg!(&hash, &value);
        LogValue {
            date_time: date_time_now(),
            hash: hash,
            value: value,
        }
    }
}

use serde::{de, Deserializer, Serializer};
pub(crate) fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTimeFix, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s +" +0300";
    let dt = DateTimeFix::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f %z").map_err(de::Error::custom)?;
    Ok(dt-Duration::hours(3))
}

pub(crate) fn date_time_to_str<S>(dt: &DateTimeFix, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
//     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    let s = (*dt+Duration::hours(3))
    .format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
    serializer.serialize_str(&s)
}

enum LoggerType {
    Json {
        sessions: Vec<json::NewJsonLog>,
    },
    CSV {
        sessions: Vec<csv::SessionTime>,
    },
}

pub struct Logger {
    log_type: LoggerType,
    
}

impl Logger {
    pub fn open_json() -> Self {
        Logger {
            log_type: LoggerType::Json {
                sessions: Vec::new(),
            },
        }
    }
    pub fn open_csv() -> Self {
        let sessions_path = get_file_path("csv/session.csv");
        Logger {
            log_type: LoggerType::CSV {
                sessions: csv::read_session_full(&sessions_path)
                    .unwrap_or(Vec::new()),
            },
        }
    }
    
    pub fn get_last_values(&self) -> Option<&Vec<crate::LogValue>> {
        match self.log_type {
        LoggerType::CSV {ref sessions} => sessions.last()?.values.as_ref(),
        LoggerType::Json {ref sessions} => Some(&sessions.last()?.values),
        _ => None,
        }
    }
    
    pub fn new_session(&mut self, values: &Vec<crate::LogValue>) {
        dbg!();
        if values.len() < 2 {return;}
        let start = values.first().unwrap().date_time;
        let finish = values.last().unwrap().date_time;
        
        match self.log_type {
        LoggerType::CSV {ref mut sessions} => {
            let s = csv::SessionTime {
                start: start,
                finish: finish,
                file_name: Some(format!("value_{}.csv", date_time_to_string_name(&start))),
                values: Some(values.clone()),
            };
            csv::write_values(&get_file_path("csv").join(s.file_name.clone().unwrap()), s.values.clone().unwrap());
            sessions.push(s);
            csv::write_session(&get_file_path("csv/session.csv"), sessions.clone());
        },
//         LoggerType::Json {ref mut sessions} => sessions.push(json::NewJsonLog {
//             start: start,
//             finish: finish,
//             values: values.clone(),
//         }),
        _ => {}
        }
    }
    
    pub fn new_table_fields(values: &Vec<crate::LogValue>, step_sec: u16, name_hash: Vec<(&str, &str)>) {
        if name_hash.len() < 1 || values.len()<2 {return;}
        
        let start = values.first().unwrap().date_time;
        
        let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
        let first_hash = name_hash.first().unwrap().1.to_owned();
        
        let cnt = values.iter().filter(|v| v.hash == first_hash).count();
//         dbg!(&dt_dlt, &cnt);
        let stp = cnt as f32 / (dt_dlt /step_sec as i32).num_seconds() as f32;
//         dbg!(&stp);
        let stp = stp as usize;
        if stp == 0 {return;}
        
        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
//         dbg!(&fields);
        
//         let name_hash = &mut name_hash;
//         name_hash.insert(0, (&"dt", &first_hash));
        
        let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
        values.iter()
            .filter(move |v| &v.hash == hash)
            .zip(0..cnt)
            .map(|(v,i)| //if name == &"dt" {
//                 format!("{1};{0}", i/stp, 
//                 (v.date_time+crate::Duration::hours(3)).format("%H:%M:%S").to_string()
//                 )
//             } else {
                format!("{:.1}", v.value)
            ).step_by(stp)
        }).collect();
        
        let lst : Vec<_> = convert::MyZip::new(lst)
            .collect();
//     dbg!(lst);

        use std::fs::OpenOptions;
        let mut wrt = ::csv::WriterBuilder::new()
//             .has_headers(true)
            .delimiter(b';')
            .from_path(
//                 &get_file_path("csv")
                PathBuf::from(r"/home/user/Рабочий стол/Graphic/Таблицы/")
                .join(format!("Table {}.csv", date_time_to_string_name_short(&start)))
                ).unwrap();
        wrt.write_record(&fields).unwrap();
//         dbg!();
        for s in lst {
            wrt.write_record(&s).unwrap();
        }
        
    }
}
