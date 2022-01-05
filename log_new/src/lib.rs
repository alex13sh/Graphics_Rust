#![allow(unused_imports)]
#![allow(dead_code)]

mod value;
mod utils;
mod files;
mod async_channel;
mod stat_info;

use value::{ValuesLine, Value};
pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;
use async_channel::{broadcast, Sender};

use std::path::PathBuf;

pub struct LogSession {
    log_dir: PathBuf,
    value_rows: Vec<ValuesLine<Value>>,
}

impl LogSession {
    pub fn new() -> Self {
        Self {
            log_dir: utils::get_file_path("log/"),
            value_rows: Vec::new(),
        }
    }
    
    pub fn relate_path(&self, rel: &str) -> PathBuf {
        self.log_dir.join(rel)
    }
    
    pub fn push_values(&mut self, values: Box<[Value]>) {
        self.value_rows.push(values.into());
    }
}
