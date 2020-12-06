 use std::fs::File;
 use std::error::Error;
 use serde::Deserialize;

 #[derive(Debug, Deserialize)]
 pub struct SessionTime {
    start: String,
    finish: String,
 }
 
 pub fn test_read_csv_1(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
    
    for result in rdr.deserialize() {
        let record:SessionTime = result?;
        println!("{:?}", record);
    }
    Ok(())
 }
