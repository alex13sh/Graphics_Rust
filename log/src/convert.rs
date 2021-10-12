#![allow(dead_code, unused_variables, unused_imports)]

use std::fs::File;
pub use std::path::PathBuf;
use std::io::prelude::*;
use std::time::Duration;
use std::collections::HashMap;

pub fn json2csv(file_name: &str, from_dir: &str, to_dir: &str) -> crate::MyResult {
    let path = crate::get_file_path(&(from_dir.to_owned() + file_name+".json"));
    let path_to = crate::get_file_path(&(to_dir.to_owned() + file_name+".csv"));
    
    let mut contents = String::new();
    let file = File::open(path)?;
//     file.read_to_string(&mut contents);
    let js: crate::json::NewJsonLog = serde_json::from_str(&contents)?;
    crate::csv::write_values(&path_to, &js.values)?;
    Ok(())
}

// #[test]
fn test_excel() -> crate::MyResult {

    let path = std::path::Path::new("/home/user/.local/share/graphicmodbus/csv/+value_09_08_2021__11_58_46_634868986_filter_0.1.xlsx");
    let book = umya_spreadsheet::reader::xlsx::read(path).unwrap();
    dbg!(book);
    assert!(false);

    Ok(())
}

