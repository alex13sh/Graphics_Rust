   
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
        use crate::async_channel::*;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
        if let Some(lines) = read_values(&format!("{}.csv", file_path)) {
            
            let lines = lines.map(|v: ValueDate<ValueOld>| 
                ValueDate {
                    date_time: v.date_time,
                    value: Value::from(v.value),
                }
            );
            
            let (mut s1, r1) = broadcast(10);
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
            let lines = values_to_line(futures::stream::iter(values));
            let lines = values_line_to_hashmap(lines);
            futures::executor::block_on( write_values_async(format!("{}_table.csv", file_path), lines) ).unwrap();
//             assert!(false);
        }
    }
    
    #[test]
    fn test_values_line_diff() {
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
        if let Some(values) = read_values::<crate::value::LogValueRawOld>(&format!("{}.csv", file_path)) {
            use crate::convert::stream::*;
            let lines = values_to_line(futures::stream::iter(values));
            let values = values_from_line_with_diff(lines);
            futures::executor::block_on( write_values_async(format!("{}_diff.csv", file_path), values) ).unwrap();
        }
    }
}

pub mod excel {
    use umya_spreadsheet::structs::*;
//     use excel::*;
    use super::inner::*;
    use crate::LogState;
    use crate::value::simple;
    
    pub struct File {
        book: Spreadsheet,
        file_path: PathBuf,
    }
    
    impl File {
        pub fn create(file_path: impl AsRef<Path>) -> Self {
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
//                 file: self,
//                 name: name,
                ws: self.book.get_sheet_by_name_mut(name).unwrap(),
            }
        }
    }
    
    pub struct Sheet<'f> {
//         file: &'f mut File,
//         name: &'static str,
        ws: &'f mut Worksheet
    }
    
    impl <'f> Sheet <'f> {
        pub async fn write_values(&mut self, pos: (usize, usize), values: impl Stream<Item=simple::ValuesMap> + std::marker::Unpin) {
            let mut values = values.enumerate().peekable();
            
            let l = &std::pin::Pin::new(&mut values).peek().await.unwrap().1;
            for (col, name) in l.values.keys().enumerate() {
                self.ws.get_cell_by_column_and_row_mut(pos.0 + col, pos.1).set_value(name);
            }
        
            while let Some((row, l)) = values.next().await {
                for (col, v) in l.values.values().enumerate() {
                    self.ws.get_cell_by_column_and_row_mut(pos.0 + col, pos.1 + row+1).set_value(v);
                }
            };
        }
        pub fn write_state(&mut self, pos: (usize, usize), state: LogState) {
            let mut fields = Vec::new();
            fields.push(("Время работы (сек)", state.time_work.to_string()));
            fields.push(("Время разгона (сек)", state.time_acel.to_string()));
            fields.push(("Обороты двигателя (об/мин)", state.hz_max.to_string()));
            fields.push(("Максимальная вибрация", state.vibro_max.to_string()));
            fields.push(("Зона вибрации (об/мин)", state.hz_vibro.to_string()));
            fields.push(("Максимальный ток", state.tok_max.to_string()));
            
            for (f, row) in fields.into_iter().zip((pos.1+1)..) {
                self.ws.get_cell_by_column_and_row_mut(1+pos.0, row).set_value(f.0);
                self.ws.get_cell_by_column_and_row_mut(2+pos.0, row).set_value(f.1);
            }
            for (f, row) in state.temps.into_iter().zip((pos.1+8)..) {
                self.ws.get_cell_by_column_and_row_mut(pos.0+1, row).set_value(f.0);
                self.ws.get_cell_by_column_and_row_mut(pos.0+2, row).set_value(format!("{:.2}", f.1.0));
                self.ws.get_cell_by_column_and_row_mut(pos.0+3, row).set_value(format!("{:.2}", f.1.1));
            }
        }
    }
    
    #[test]
    fn test_convert_csv_raw_to_excel() {
        use crate::convert::stream::*;
        use crate::async_channel::*;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
        if let Some(values) = super::csv::read_values(&format!("{}.csv", file_path)) {
            let values = raw_to_elk(values);
            let lines = values_to_line(futures::stream::iter(values)).take(100);
            
            let (s, l1) = broadcast(10);
            
//             let mut lines = lines.boxed();
//             let f1 = async move {
//                 while let Some(l) = lines.next().await {
//                     dbg!(&l);
//                     s.send(l).await;
//                 }
//                 dbg!("s close");
//                 drop(s);
//             };
//             let f1 = lines.for_each(move |l| async {s.send(l).await;});
            let f1 = lines.map(|l| Ok(l)).forward(s);
            let l2 = l1.clone();
            let f2 = async move {
                
                let l1 = values_line_to_hashmap(l1);
                let mut f = File::create(format!("{}.xlsx", file_path));
//                 {
                    let l2 = values_line_to_simple(l2);
                    let mut s = f.open_sheet("Sheet1");
                    s.write_values((1,1), l1).await;
                    
                    let stat = crate::stat_info::simple::calc(l2).fold(None, |_, s| async{Some(s)}).await;
                    s.write_state((17,2), stat.unwrap());
                f.save();
            };
            
            futures::executor::block_on(futures::future::join(f1, f2));
        }
    }
}

mod inner {
    pub use futures::stream::{Stream, StreamExt};
    pub use std::path::{PathBuf, Path};
    pub use std::fs::File;
}
