
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

pub mod json;
pub mod csv;
pub use json::*;
// pub use csv::*;

pub(crate) fn get_file_path(file_name: &str) -> PathBuf {
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
    pub date_time: String,
    pub hash: String,
    pub value: f32,
}

