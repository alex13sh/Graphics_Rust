#![feature(explicit_generic_args_with_impl_trait)]

#![allow(unused_imports)]
#![allow(dead_code)]

mod value;
mod utils;
mod files;
mod async_channel;
mod stat_info;
mod convert;

use value::{ValuesLine, Value};
use stat_info::simple::LogState;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;

use std::path::PathBuf;

pub struct LogSession {
    log_dir: PathBuf,
    // value_rows: Vec<ValuesLine<Value>>,
    values_elk: async_channel::Sender<value::ElkValuesLine>,
    values_raw: async_channel::Sender<value::ValuesLine<value::Value>>,
}

impl LogSession {
    pub fn new() -> Self {
        Self {
            log_dir: utils::get_file_path("log/"),
            values_elk: async_channel::broadcast(20).0,
            values_raw: async_channel::broadcast(20).0,
        }
    }
    
    pub fn relate_path(&self, rel: &str) -> PathBuf {
        self.log_dir.join(rel)
    }
    
    pub async fn push_values(&mut self, values: Box<[Value]>) {
        use async_channel::*;
        use convert::value::*;
        // self.value_rows.push(values.into());
        let line = value::ValuesLine::from(values);

        self.values_elk.send(values_line_convert(line.clone())).await.unwrap();
        self.values_raw.send(line).await.unwrap();
    }

    pub async fn write_excel(&self) {
        files::excel::write_file(&self.log_dir, self.values_elk.subscribe()).await;
    }

    pub async fn write_csv_elk(&self) {
        use convert::stream::*;
        use files::csv::*;
        let values = values_from_line(self.values_elk.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }
    pub async fn write_csv_raw(&self) {
        use convert::stream::*;
        use files::csv::*;
        let values = values_from_line(self.values_raw.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }
    pub async fn write_csv_raw_diff(&self) {
        use convert::stream::*;
        use files::csv::*;
        let values = values_from_line_with_diff(self.values_raw.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }

    pub async fn write_full(&self) {
        futures::join!(
            self.write_csv_elk(),
            self.write_excel(),
            self.write_csv_raw(),
            self.write_csv_raw_diff(),
        );
    }
}
