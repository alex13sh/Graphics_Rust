use std::fs::File;
use std::io::prelude::*;

pub fn json2csv(file_name: &str, from_dir: &str, to_dir: &str) -> std::io::Result<()> {
    let path = crate::get_file_path(&(from_dir.to_owned() + file_name));
    
    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents);
    
    Ok(())
}
