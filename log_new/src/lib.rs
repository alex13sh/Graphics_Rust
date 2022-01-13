#![feature(explicit_generic_args_with_impl_trait)]

#![allow(unused_imports)]
#![allow(dead_code)]

mod value;
pub mod utils;
mod files;
mod async_channel;
mod stat_info;
mod convert;

pub use value::{ValuesLine, Value};
pub use utils::get_file_path;
pub use stat_info::simple::LogState;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;

use std::{path::PathBuf, sync::Arc};
use futures::{Stream, StreamExt};
use futures::stream::BoxStream;

pub struct LogSession {
    log_dir: PathBuf,
    // value_rows: Vec<ValuesLine<Value>>,
    values_elk: Option<async_channel::Sender<value::ElkValuesLine>>,
    values_raw: Option<async_channel::Sender<value::ValuesLine<value::Value>>>,
}

impl LogSession {
    pub fn new() -> Self {
        Self {
            log_dir: utils::get_file_path("log/"),
            values_elk: None,
            values_raw: None,
        }
    }

    pub fn start(&mut self) {
        self.values_elk = Some(async_channel::broadcast(20).0);
        self.values_raw = Some(async_channel::broadcast(20).0);
    }
    pub fn stop(&mut self) {
        self.values_elk = None;
        self.values_raw = None;
    }
    
    pub fn relate_path(&self, rel: &str) -> PathBuf {
        self.log_dir.join(rel)
    }
    
    pub fn push_values(&mut self, values: Box<[Value]>) {
        if let (Some(ref mut elk), Some(ref mut raw)) 
            = (self.values_elk.as_mut(), self.values_raw.as_mut()) {

            use async_channel::*;
            use convert::value::*;
            // self.value_rows.push(values.into());
            let line = value::ValuesLine::from(values);

            let f = async {
                elk.send(values_line_convert(line.clone())).await.unwrap();
                // raw.send(line).await.unwrap();
            };
            futures::executor::block_on(f);
        }
    }

    pub async fn write_excel(&self) {
        let elk = self.values_elk.as_ref().unwrap();
        files::excel::write_file(&self.log_dir, elk.subscribe()).await;
    }

    pub async fn write_csv_elk(&self) {
        use convert::stream::*;
        use files::csv::*;
        let elk = self.values_elk.as_ref().unwrap();
        let values = values_from_line(elk.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }
    pub async fn write_csv_raw(&self) {
        use convert::stream::*;
        use files::csv::*;
        let raw = self.values_raw.as_ref().unwrap();
        let values = values_from_line(raw.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }
    pub async fn write_csv_raw_diff(&self) {
        use convert::stream::*;
        use files::csv::*;
        let raw = self.values_raw.as_ref().unwrap();
        let values = values_from_line_with_diff(raw.subscribe());
        write_values_async(&self.log_dir, values).await.unwrap();
    }

    pub async fn write_full(self: Arc<Self>) {
        futures::join!(
            self.write_csv_elk(),
            self.write_excel(),
            self.write_csv_raw(),
            self.write_csv_raw_diff(),
        );
    }

    pub fn get_statistic_low(&self) -> Option<BoxStream<'static, stat_info::simple::LogState>>{
        let elk = self.values_elk.as_ref()?;
        let lines = crate::stat_info::simple::filter_half_low(elk.subscribe());
        Some(stat_info::simple::calc(lines).boxed())
    }
    pub fn get_statistic_top(&self) -> Option<BoxStream<'static, stat_info::simple::LogState>>{
        let elk = self.values_elk.as_ref()?;
        let lines = crate::stat_info::simple::filter_half_top(elk.subscribe());
        Some(stat_info::simple::calc(lines).boxed())
    }
}
