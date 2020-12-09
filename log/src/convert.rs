use std::fs::File;
use std::io::prelude::*;

pub fn json2csv(file_name: &str, from_dir: &str, to_dir: &str) -> crate::MyResult {
    let path = crate::get_file_path(&(from_dir.to_owned() + file_name+".json"));
    let path_to = crate::get_file_path(&(to_dir.to_owned() + file_name+".csv"));
    
    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents);
    let js: crate::json::NewJsonLog = serde_json::from_str(&contents)?;
    crate::csv::write_values(path_to, js.values)?;
    Ok(())
}
