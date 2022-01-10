   
pub mod csv {
    use super::inner::*;
    use csv::WriterBuilder;

    pub fn read_values<T>(file_name: impl AsRef<Path>) -> Option<impl Iterator<Item=T>>
    where T: for<'de> serde::Deserialize<'de>
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
    
    pub fn write_values<T>(file_name: impl AsRef<Path>, values: impl Iterator<Item=T>) -> crate::MyResult 
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
    
    pub async fn write_values_async<T>(file_name: impl AsRef<Path>, values: impl Stream<Item=T> ) -> crate::MyResult 
    where T: serde::Serialize
    {
        let file = File::create(file_name)?;
        let wrt = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_writer(file);
        
//         while let Some(value) = values.next().await {
//             wrt.serialize(value)?;
//         }
        values.fold(wrt, |mut wrt, v| async {
            wrt.serialize(v).unwrap();
            wrt
        }).await;
        
        Ok(())
    }
    
    #[test]
    fn test_read_write_csv() {
        use crate::value::raw::*;
        use crate::value::ValueDate;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
        if let Some(lines) = read_values(&format!("{}.csv", file_path)) {
            
            let lines = lines.map(|v: ValueDate<ValueOld>| 
                ValueDate {
                    date_time: v.date_time,
                    value: Value::from(v.value),
                }
            );
            
            let (s1, r1) = crate::broadcast(10);
            let r2 = r1.clone();
//             write_values("/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376_sync.csv", lines).unwrap();
            let f0 = async move {
                for v in lines {
                    dbg!(&v);
                    s1.send(v).await;
                }
            };
            let f1 = write_values_async(format!("{}_async_1.csv", file_path), r1);
            let f2 = write_values_async(format!("{}_async_2.csv", file_path), r2);
            
            let _ = futures::executor::block_on( futures::future::join3(f0, f1, f2) );
            
        } else {
            assert!(false);
        }
//         assert!(false);
    }
    
    #[test]
    fn test_convert_raw_to_elk() {
        use crate::value::raw::*;
        use crate::value::ValueDate;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
        if let Some(values) = read_values(&format!("{}.csv", file_path)) {
            let values = crate::convert::stream::raw_to_elk(values);
            write_values(&format!("{}_elk.csv", file_path), values).unwrap();
        }
//         assert!(false);
    }
    
    #[test]
    fn test_convert_to_table() {
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
        if let Some(values) = read_values(&format!("{}.csv", file_path)) {
            use crate::convert::stream::*;
            let values = crate::convert::stream::raw_to_elk(values);
            let lines = values_to_line(futures::stream::iter(values), 0.1);
            let lines = values_line_to_hashmap(lines);
            futures::executor::block_on( write_values_async(format!("{}_table.csv", file_path), lines) ).unwrap();
            assert!(false);
        }
    }
}

pub mod excel {
    //use umya_spreadsheet::*;
//     use excel::*;
    use super::inner::*;
    
    pub struct File {
        book: umya_spreadsheet::structs::Spreadsheet,
        file_path: PathBuf,
    }
    
    impl File {
        pub fn create(file_path: impl AsRef<PathBuf>) -> Self {
            Self {
                book: umya_spreadsheet::new_file(),
                file_path: file_path.as_ref().into(),
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
        pub fn write_value<T>(&mut self, _pos: (u16, u16), _values: Vec<T>) 
        where T: serde::Serialize
        {
        
        }
    }
}

mod inner {
    pub use futures::stream::{Stream, StreamExt};
    pub use std::path::{PathBuf, Path};
    pub use std::fs::File;
}
