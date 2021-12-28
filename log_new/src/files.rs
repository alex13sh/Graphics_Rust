
pub mod csv {
    pub fn read_values< T>(file_name: &PathBuf) -> Option<Vec<T>>
    where T:  for<'de> serde::Deserialize<'de>
    {
        let file = File::open(file_name).ok()?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(file);
        
        Some(rdr.deserialize()
            .filter_map(|res| res.ok())
            .collect()
        )
    }
    
    pub fn write_values<T>(file_name: &PathBuf, values: &Vec<T>) -> crate::MyResult 
    where T: serde::Serialize
    {
        let file = File::create(file_name)?;
        let mut wrt = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_writer(file);
        
        for value in values {
            wrt.serialize(value)?;
        }
        
        Ok(())
    }
}

pub mod excel {
    //use umya_spreadsheet::*;
    use excel::*;
    
    pub struct File {
        book: // umya_spreadsheet::new_file(),
        file_path: Path,
    }
    
    impl File {
        pub fn create(file_path: &PathBuf) -> Self {
            Self {
                book: umya_spreadsheet::new_file(),
                file_path: file_path.into(),
            }
        }
        pub fn save(&self) {
            let _ = umya_spreadsheet::writer::xlsx::write(&self.book, &self.file_path);
        }
        pub fn open_sheet(&mut self, name: &'static str) -> Sheet {
            Sheet {
                file: self,
                name: name,
            }
        }
    }
    
    pub struct Sheet<'f> {
        file: &'f mut File,
        name: &'static str,
        
    }
    
    impl <'f> Sheet <'f> {
        pub fn write_value<T>(&mut self, pos: (u16, u16), values: Vec<T>) 
        where T: serde::Serialize
        {
        
        }
    }
}
