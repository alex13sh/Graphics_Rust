   
pub mod csv {
    use super::inner::*;
    use csv::WriterBuilder;

    pub fn read_values< T>(file_name: &PathBuf) -> Option<impl Iterator<Item=T>>
    where T:  for<'de> serde::Deserialize<'de>
    {
        
        let file = File::open(file_name).ok()?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(file);
            
       let itr =  std::iter::from_fn(move || {
            rdr.deserialize()
                .filter_map(|res| res.ok())
            }.next()
        );
        Some(itr)
    }
    
    pub fn write_values<T>(file_name: &PathBuf, values: impl Iterator<Item=T>) -> crate::MyResult 
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
    
    pub async fn write_values_async<T>(file_name: &PathBuf, values: impl Stream<Item=T>) -> crate::MyResult 
    where T: serde::Serialize
    {
        let file = File::create(file_name)?;
        let mut wrt = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_writer(file);
        
        while let Some(value) = values.next().await {
            wrt.serialize(value)?;
        }
        
        Ok(())
    }
}

pub mod excel {
    //use umya_spreadsheet::*;
//     use excel::*;
    use super::inner::*;
    
    pub struct File {
        book: umya_spreadsheet::structs::Spreadsheet,
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

mod inner {
    pub use futures::stream::{Stream, StreamExt};
    pub use std::path::PathBuf;
    pub use std::fs::File;
}
