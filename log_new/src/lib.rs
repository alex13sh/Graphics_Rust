#![feature(explicit_generic_args_with_impl_trait)]
#![feature(result_option_inspect)]

#![allow(unused_imports)]
#![allow(dead_code)]

mod value;
pub mod utils;
mod files;
mod async_channel;
mod stat_info;
mod convert;

use utils::DateTimeFix;
pub use value::{ValuesLine, Value};
pub use utils::get_file_path;
pub use stat_info::simple::LogState;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;

use std::{path::PathBuf, sync::Arc};
use futures::{Stream, StreamExt};
use futures::stream::BoxStream;
use std::future::Future;


#[derive(Clone)]
pub struct LogSession {
    log_dir: PathBuf,
    date_time: DateTimeFix,
    // value_rows: Vec<ValuesLine<Value>>,
    values_elk: Option<async_channel::Sender<value::ElkValuesLine>>,
    values_raw: Option<async_channel::Sender<value::ValuesLine<value::Value>>>,
}

impl LogSession {
    pub fn new() -> Self {
        Self {
            log_dir: utils::get_file_path("log/values/"),
            date_time: utils::date_time_now(),
            values_elk: None,
            values_raw: None,
        }
    }

    pub fn start(&mut self) {
        self.values_elk = Some(async_channel::broadcast(20).0);
        self.values_raw = Some(async_channel::broadcast(20).0);
        self.date_time = utils::date_time_now();
    }
    pub fn stop(&mut self) {
        self.values_elk = None;
        self.values_raw = None;
    }
    
    pub fn relate_path(&self, rel: &str) -> PathBuf {
        self.log_dir.join(rel)
    }
    fn date_time_str(&self) -> String {
        utils::date_time_to_string_name_short(&self.date_time)
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
                raw.send(line).await.unwrap();
            };
            futures::executor::block_on(f);
        }
    }

    pub fn make_path_excel(&self, suffix: &str) -> PathBuf {
        self.log_dir
        .join("excel").join(&format!("{}_{}", self.date_time_str(), suffix))
        .with_extension("xlsx")
    }
    pub fn write_excel_low(&self) -> impl Future<Output=()> {
        let elk = self.values_elk.as_ref().unwrap();
        let file_path = self.make_path_excel("low");

        let values_line = elk.subscribe();
        let values_low = crate::stat_info::simple::filter_half_low(values_line);
        files::excel::write_file(file_path, values_low)
    }
    pub fn write_excel_top(&self) -> impl Future<Output=()> {
        let elk = self.values_elk.as_ref().unwrap();
        let file_path = self.make_path_excel("top");

        let values_line = elk.subscribe();
        let values_top = crate::stat_info::simple::filter_half_top(values_line);
        files::excel::write_file(file_path, values_top)
    }

    pub fn write_csv_elk(&self) -> impl Future<Output=()> {
        use convert::stream::*;
        use files::csv::*;
        let elk = self.values_elk.as_ref().unwrap();
        let values = values_from_line(elk.subscribe());
        let file_path = self.log_dir
            .join("csv").join(&self.date_time_str())
            .with_extension("csv");
        write_values_async(file_path,
            values).unwrap()
    }
    pub fn write_csv_raw(&self) -> impl Future<Output=()> {
        use convert::stream::*;
        use files::csv::*;
        let raw = self.values_raw.as_ref().unwrap();
        let values = values_from_line(raw.subscribe());
        let file_path = self.log_dir
            .join("csv_raw").join(&self.date_time_str())
            .with_extension("csv");
        write_values_async(file_path, 
            values).unwrap()
    }
    pub fn write_csv_raw_diff(&self) -> impl Future<Output=()> {
        use convert::stream::*;
        use files::csv::*;
        let raw = self.values_raw.as_ref().unwrap();
        let values = values_from_line_with_diff(raw.subscribe());
        let file_path = self.log_dir
            .join("csv_raw").join(&format!("{}_diff", self.date_time_str()))
            .with_extension("csv");
        write_values_async(file_path, 
            values).unwrap()
    }

    pub fn write_full(&self) -> impl Future<Output=()> {
//         let f = futures::future::join_all(vec![
//             self.write_csv_elk(),
//             self.write_excel(),
//             self.write_csv_raw(),
//             self.write_csv_raw_diff(),
//         ]);
//         async {
//             let _ = f.await;
//         }
        let f1 = self.write_csv_elk();
        let f2l = self.write_excel_low();
        let f2t = self.write_excel_top();
        let f3 = self.write_csv_raw();
        let f4 = self.write_csv_raw_diff();
        async move {
            futures::join!(
                f1,
                f2l, f2t,
                f3, f4
            );
        }
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
