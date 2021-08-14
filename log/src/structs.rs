#![allow(dead_code, unused_variables, unused_imports)]

use std::fs::File;
pub use std::path::PathBuf;
use std::io::prelude::*;
use std::time::Duration;
use std::collections::HashMap;

pub struct Converter {
    input_path_csv: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) file_name: String,
}

pub struct InputValues {
    pub(crate) converter: Option<Converter>,
    name_hash: Vec<(String, String)>,
    values: Vec<crate::LogValue>
}

pub struct OutputValues {
    pub(crate) converter: Option<Converter>,
    info: TableInfo,
    pub fields: Vec<String>,
    pub values: ValuesF,
}

struct TableInfo {
    step_sec: f32, // Шаг времени
    count: u32, // Кол-во значений
}

pub struct TableState {
    pub time_work: f32, // время работы // count * step_sec = time
    pub time_acel: f32, // Время разгона
    pub hz_vibro: u32, // Зона вибрации
}

impl Converter {
    pub fn new(input_path_csv: PathBuf, output_path: PathBuf) -> Self {
        Converter {
            input_path_csv: input_path_csv,
            output_path: output_path,
            file_name: String::new(),
        }
    }

    pub fn from_log_values(self, values: Vec<crate::LogValue>) -> InputValues {
        InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(values)
        }
    }
    pub fn read_file(mut self, file_name: &str, reader: fn(&PathBuf) -> Vec<crate::LogValue>) -> InputValues {
        self.file_name = file_name.to_owned();
        let cur_path = self.input_path_csv.join(file_name.to_owned()+".csv");
        InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(reader(&cur_path))
        }
    }
    pub fn read_file_opt(mut self, file_name: &str, reader: fn(&PathBuf) -> Option<Vec<crate::LogValue>>) -> Option<InputValues> {
        self.file_name = file_name.to_owned();
        let cur_path = self.input_path_csv.join(file_name.to_owned()+".csv");
        let v = InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(reader(&cur_path)?)
        };
        Some(v)
    }
    pub fn output_file(output_path: PathBuf, file_name: &str) -> Self {
        Converter {
            input_path_csv: PathBuf::new(),
            output_path: output_path,
            file_name: file_name.to_owned(),
        }
    }
}

impl InputValues {
    pub fn from_log_values(values: Vec<crate::LogValue>) -> InputValues {
        InputValues {
            converter: None,
            values: values,
            name_hash: Vec::new(),
        }
    }
    
    pub fn fields(mut self, name_hash: Vec<(&str, &str)>) -> Self {
        self.name_hash = name_hash.into_iter()
            .map(|(name, hash)| (name.into(), hash.into()))
            .collect();
        self
    }
    
    pub fn make_values_3(self, step_sec: Duration) -> OutputValues {
        let values = &self.values;
        let name_hash = &self.name_hash;

        let dt_finish = values.last().unwrap().date_time;
        let dt_start = values.first().unwrap().date_time;
        let dt_dlt =  dt_finish - dt_start;
        let cnt = values.iter().filter(move |v| v.hash == "4bd5c4e0a9").count();
        dbg!(&dt_dlt, &cnt);
        let dt_dlt: Duration = dt_dlt.to_std().unwrap();
        let step_ms = step_sec.as_millis();
        let cnt = dt_dlt.as_millis() / step_ms;
        let cnt = cnt as u32;
        dbg!(&cnt);

    //Vec::with_capacity(name_hash.len())
        let mut values_f32 = ValuesF::from_size(name_hash.len(), cnt as usize, -13.37);
        let fields: HashMap<_, _> = name_hash.iter().zip(0..).map(|((_,hash), i)| ( hash.to_owned(), i)).collect();
        dbg!(&fields);
    //     let step_ms = step_sec.as_millis() as i64;
        for v in values {
            let i = (v.date_time.timestamp_millis() - dt_start.timestamp_millis())/(step_ms as i64);
            if let Some(f) = fields.get(&v.hash) {
    //             dbg!(i, *f);
                values_f32.0[i as usize][*f as usize] = v.value;
            }
        }

//         let mut values: Vec<Vec<String>> = std::iter::repeat(vec![String::from("");name_hash.len()+1]).take(cnt as usize+1).collect();
        let step_sec_f = step_sec.as_secs_f32();
//         let values_str: Vec<_> = values_f32.into_iter().zip(0..)
//             .map(|(row, i)| {
//                 let time = i as f32 * step_sec_f;
//                 let mut rows = Vec::new();
//                 rows.push(format!("{:.1}", time).replace(".", ","));
//                 rows.extend(row.into_iter().map(|v|
//                     if v == -13.37 { String::new()}
//                     else { format!("{:.2}", v).replace(".", ",")}
//                 ));
//                 rows
//             }).collect();
//         let values_str = values_f32.to_string();

        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt,
                step_sec: step_sec.as_secs_f32(),
            },
            fields: fields,
//             values_str: values_str,
            values: values_f32,
        }
    }
}

impl OutputValues {
    pub fn get_state(&self) -> TableState {
        let time_work = self.info.count as f32 / self.info.step_sec;
        
        TableState {
            time_work: time_work,
            time_acel: 0.0,
            hz_vibro: 0,
        }
    }

    pub fn write_csv(self) -> crate::MyResult {
        let conv = &self.converter.ok_or("Converter is empty")?;
        let info = &self.info;
        let new_path = conv.output_path
            .join(format!("{}_filter_{}.csv", conv.file_name, info.step_sec));
        use std::fs::OpenOptions;
        let mut wrt = ::csv::WriterBuilder::new()
            .delimiter(b';')
    //         .from_writer(file);
            .from_path(new_path)?;
        wrt.write_record(&self.fields).unwrap();

        let values_str = self.values.to_string();
        for s in values_str {
            if !s[0].is_empty() {
                wrt.write_record(&s)?;
            }
        }
        Ok(())
    }

    pub fn write_excel(mut self) -> crate::MyResult {
//         use umya_spreadsheet::*;
        let conv = &self.converter.ok_or("Converter is empty")?;
        let info = &self.info;
        let new_path = conv.output_path
            .join(format!("{}_filter_{}.xlsx", conv.file_name, info.step_sec));
        let mut book = umya_spreadsheet::new_file();
//         let sht = book.new_sheet("Лог")?;
        let sht = book.get_sheet_by_name_mut("Sheet1")?;

        self.fields.insert(0, "time".into());
//         self.values = self.values.insert_column(0, (0..info.count).map(|v| v as f32 *info.step_sec));
        let values_str = self.values.to_string()
            .insert_column(0, (0..info.count).map(|v| {
                let v = v as f32 *info.step_sec;
                format!("{:.1}", v)
            }));

        for (f, col) in self.fields.iter().zip(1..) {
            sht.get_cell_by_column_and_row_mut(col, 1).set_value(f);
        }

        for (s, row) in values_str.into_iter()
//             .filter(|s| !s[0].is_empty())
            .zip(2..) {

            for (v, col) in s.iter().zip(1..) {
                if v.is_empty() {
                    let v = sht.get_cell_by_column_and_row(col, row-1).ok_or("Error Cell")?.get_value().clone();
                    sht.get_cell_by_column_and_row_mut(col, row).set_value(v);
                } else {
                    sht.get_cell_by_column_and_row_mut(col, row).set_value(v);
                }
            }
        }

        let _ = umya_spreadsheet::writer::xlsx::write(&book, &new_path);
        Ok(())
    }

}

pub use inner::*;
mod inner {

    pub type ValuesF = ValuesMat<f32>;
    pub type ValuesS = ValuesMat<String>;

    pub struct ValuesMat<T>(pub Vec<Vec<T>>);

    impl <T> ValuesMat <T>
    where T: std::clone::Clone
    {
        pub fn from_size(cols: usize, rows: usize, value: T) -> Self {
            let v = std::iter::repeat(vec![value;cols]).take(rows+1).collect();
            ValuesMat(v)
        }
        pub fn insert_column(self, col: usize, values: impl Iterator<Item=T>) -> Self {
            if self.0.is_empty() {return ValuesMat(Vec::new());}
            let rows = self.0[0].len();
            let v = self.0.into_iter()
                .zip(values)
                .map(|(row, v)| {
                    let mut row_new = Vec::with_capacity(rows+1);
                    row_new.extend(row.into_iter());
                    row_new.insert(col, v);
                    row_new
                }).collect();
            ValuesMat(v)
        }
    }

    impl ValuesMat<f32> {
        pub fn to_string(&self) -> ValuesMat<String> {
            let v = self.0.iter()
//             .zip(0..)
//             .map(|(row, i)| {
            .map(|row| {
                let mut rows = Vec::new();
//                 let time = i as f32 * step_sec_f;
//                 rows.push(format!("{:.1}", time).replace(".", ","));
                rows.extend(row.into_iter().map(|&v|
                    if v == -13.37 { String::new()}
                    else { format!("{:.2}", v).replace(".", ",")}
                ));
                rows
            }).collect();
            ValuesMat(v)
        }
    }

    impl <T> IntoIterator for ValuesMat<T> {
        type Item = <Vec<Vec<T>> as IntoIterator>::Item;
        type IntoIter = <Vec<Vec<T>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    pub struct MyZip <T, U>
    where T: Iterator<Item=U>
    {
        vec: Vec<T>
    }
    // impl <T, U> MyZip<T, U>
    // where
    //     T: IntoIterator<Item=U>
    impl <T, U> MyZip<T, U>
    where T: Iterator<Item=U>
    {
        pub fn new(v: Vec<T>) -> Self
        {
            MyZip {
                vec:v.into_iter()
                .fold(Vec::new(), |mut v, i| {
                    v.push(i);
                    v
                })
            }
        }
    }

    impl <T, U> Iterator for MyZip<T, U>
    where T: Iterator<Item=U>
    {
        type Item = Vec<U>;
        fn next(&mut self) -> Option<Self::Item> {
            self.vec.iter_mut()
                .map(|v| v.next())
                .collect()
        }
    }
    pub fn myzip<T, U>(vec: Vec<T>) -> MyZip<T::IntoIter, U>
    where
        T: IntoIterator<Item=U>
    {
        MyZip::new(vec.into_iter()
            .map(|v| v.into_iter())
            .collect())
    }

//     fn convert_cols_to_rows<T>(cols: Vec<Vec<T>>) -> Vec<Vec<T>> {
//         let mut itr = Box<dyn Iterator>;
//         for c in cols {
//             itr = itr.interleave(c);
//         }
//         itr.chunks(cols.len()).collect()
//     }

    use std::iter::Peekable;
    pub struct InsertByStep <T, I, ITP, P>
    where I: Iterator<Item=T>,
        ITP: Iterator<Item=(T,usize)>,
        P: Iterator<Item=usize>
    {
        a: Peekable<ITP>, b: I,
        pos: Peekable<P>,
    }

    impl <T, I, ITP, P> Iterator for InsertByStep <T, I, ITP, P>
        where I: Iterator<Item=T>,
        ITP: Iterator<Item=(T,usize)>,
        P: Iterator<Item=usize>
    {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            let pos_a = self.a.peek()?.1;
            let pos_p = self.pos.peek()?;
            if pos_a == *pos_p {
                self.pos.next();
                self.b.next()
            } else {
                self.a.next().map(|a| a.0)
            }
        }
    }

    pub fn insert_by_step<T, I, P>(a: I, b: I, pos: P) -> InsertByStep<T, I, impl Iterator<Item=(T,usize)>, P>
    where I: Iterator<Item=T>,
        P: Iterator<Item=usize>
    {
        InsertByStep {
            a: a.zip(0..).peekable(), b: b,
            pos: pos.peekable(),
        }
    }


    #[test]
    fn test_iner_step() {
//         use itertools::Itertools;
        let a = (0..10);
        let b = (20..25);

        let mut c = insert_by_step(a, b, (0..).step_by(2));

        assert_eq!(c.next().unwrap(), 20);
        assert_eq!(c.next().unwrap(), 0);
        assert_eq!(c.next().unwrap(), 1);
        assert_eq!(c.next().unwrap(), 21);
        assert_eq!(c.next().unwrap(), 2);
        assert_eq!(c.next().unwrap(), 3);
        assert_eq!(c.next().unwrap(), 22);
        assert_eq!(c.next().unwrap(), 4);
        assert_eq!(c.next().unwrap(), 5);
        assert_eq!(c.next().unwrap(), 23);
        assert_eq!(c.next().unwrap(), 6);
        assert_eq!(c.next().unwrap(), 7);
        assert_eq!(c.next().unwrap(), 24);
        assert_eq!(c.next().unwrap(), 8);
        assert_eq!(c.next().unwrap(), 9);
    }
}

// Всё ненужное

impl InputValues {
    pub fn make_values_1(self, step_sec: Duration) -> crate::MyResult<OutputValues> {
        let values = &self.values;
        let name_hash = &self.name_hash;
        let step_sec = step_sec.as_secs_f32();

        let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
        let cnt = values.iter().filter(move |v| v.hash == "4bd5c4e0a9").count();
        dbg!(&dt_dlt, &cnt);
        let step_ms_100 = (step_sec * 100.0) as i32;
        let dt_dlt = dt_dlt * 100;
        let stp = cnt as f32 / (dt_dlt /step_ms_100).num_seconds() as f32;
        dbg!(&stp);
        let stp = stp.round() as usize;
        if stp == 0 {dbg!(&stp);return Err("Step = 0".into());}

        let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
            values.iter()
                .filter(move |v| &v.hash == hash)
                .zip(0..cnt)
                .map(move |(v,i)|
                    v.value
                ).step_by(stp)
        }).collect();

        let lst : Vec<_> = MyZip::new(lst)
            .collect();
    //     dbg!(&lst);
        let cnt = lst.len();
        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        Ok(OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt as u32,
                step_sec: step_sec,
            },
            fields: fields,
            values: ValuesMat(lst),
        })
    }

    pub fn make_values_2(self, step_sec: Duration) -> crate::MyResult<OutputValues> {
        let values = &self.values;
        let name_hash = &self.name_hash;

        let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
//         dbg!(&dt_dlt);
        use std::collections::BTreeMap;
    //     let map_values = BTreeMap::new();

        let felds: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
        let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
            let dt_value: BTreeMap<_, _> = values.iter()
                .filter(move |v| &v.hash == hash)
                .map(move |v| {
                    let dt = (v.date_time+crate::Duration::hours(3)).format("%H:%M:%S").to_string();
                    (dt, v.value)
                }).collect();
            let val: Vec<_> = dt_value.values().cloned().collect();
            val
        }).collect();

        let lst : Vec<_> = myzip(lst)
            .collect();
        let cnt = lst.len();
        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        Ok(OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt as u32,
                step_sec: step_sec.as_secs_f32(),
            },
            fields: fields,
            values: ValuesMat(lst),
        })
    }
}
