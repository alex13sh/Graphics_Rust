
pub use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
pub use chrono::{SecondsFormat, Offset, FixedOffset, Duration};

#[derive(Clone, std::fmt::Debug)]
struct MSK;
impl Offset for MSK {
    /// Returns the fixed offset from UTC to the local time stored.
    fn fix(&self) -> FixedOffset {
        FixedOffset::east(3*60*60)
    }
}

type DateTimeLocal = chrono::DateTime<chrono::Local>;
type DateTimeFix = chrono::DateTime<chrono::FixedOffset>;
type DateTimeMSK = chrono::DateTime<MSK>;
type DateTime = DateTimeFix;

pub fn date_time_now() -> DateTime {
    DateTime::from(chrono::Local::now())
//         .east(3*60*60)
}

pub fn date_time_to_string_name(dt: &DateTime) -> String {
    dt.format("%d_%m_%Y__%H_%M_%S_%.f")
        .to_string().replace("_.", "_")
}

pub mod json;
pub mod csv;

#[cfg(feature = "convert")]
pub mod convert;

// pub use json::*;
// pub use csv::*;

pub(crate) type MyResult = Result<(), Box<dyn std::error::Error>>;

pub fn get_file_path(file_name: &str) -> PathBuf {
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
    let dt = DateTimeFix::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").map_err(de::Error::custom)?;
    Ok(dt-Duration::hours(3))
}

pub(crate) fn date_time_to_str<S>(dt: &DateTimeFix, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
//     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    let s = (*dt+Duration::hours(3))
    .format("%Y-%m-%dT%H:%M:%S%.f").to_string();
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
}
