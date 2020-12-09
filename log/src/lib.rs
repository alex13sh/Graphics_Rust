
pub use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
pub use chrono::{NaiveDateTime, SecondsFormat};

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
    #[serde(deserialize_with = "naive_date_time_from_str")]
    #[serde(serialize_with = "naive_date_time_to_str")]
    pub date_time: NaiveDateTime,
    pub hash: String,
    pub value: f32,
}

use serde::{de, Deserializer, Serializer};
pub(crate) fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").map_err(de::Error::custom)
}

pub(crate) fn naive_date_time_to_str<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
//     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    let s = dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string();
    serializer.serialize_str(&s)
}
