 use std::fs::File;
 use std::error::Error;
 use serde::{de, Deserialize, Deserializer};
 
type DateTime = chrono::DateTime<chrono::Local>;
type DateTimeFix = chrono::DateTime<chrono::FixedOffset>; 
use chrono::Duration;
use chrono::NaiveDateTime;

fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").map_err(de::Error::custom)
}

 #[derive(Debug, Deserialize)]
 pub struct SessionTime {
    #[serde(deserialize_with = "naive_date_time_from_str")]
    start: NaiveDateTime,
    #[serde(deserialize_with = "naive_date_time_from_str")]
    finish: NaiveDateTime,
 }
 
 pub fn test_read_csv_1(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file_path = super::get_file_path(file_path);
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
