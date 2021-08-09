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
    let mut file = File::open(path)?;
//     file.read_to_string(&mut contents);
    let js: crate::json::NewJsonLog = serde_json::from_str(&contents)?;
    crate::csv::write_values(&path_to, js.values)?;
    Ok(())
}

pub struct Converter {
    input_path_csv: PathBuf,
    output_path: PathBuf,
    file_name: String,
    name_hash: Vec<(String, String)>,
}

pub struct InputValues {
    converter: Converter,
    values: Vec<crate::LogValue>
}

pub struct OutputValues {
    converter: Converter,
    info: TableInfo,
    fields: Vec<String>,
    values: Vec<Vec<String>>,
}

struct TableInfo {
    step_sec: f32, // Шаг времени
    count: u32, // Кол-во значений
//     time_work: f32, // время работы // count * step_sec = time
//     time_acel: f32, // Время разгона
//     hz_vibro: u32, // Зона вибрации
}

impl Converter {
    pub fn new(input_path_csv: PathBuf, output_path: PathBuf) -> Self {
        Converter {
            input_path_csv: input_path_csv,
            output_path: output_path,
            name_hash: Vec::new(),
            file_name: String::new(),
        }
    }

    pub fn fields(mut self, name_hash: Vec<(&str, &str)>) -> Self {
        self.name_hash = name_hash.into_iter()
            .map(|(name, hash)| (name.into(), hash.into()))
            .collect();
        self
    }

    pub fn read_values(mut self, file_name: &str) -> crate::MyResult<InputValues> {
        self.file_name = file_name.to_owned();
        let cur_path = self.input_path_csv.join(file_name.to_owned()+".csv");
        let values = crate::csv::read_values(&cur_path).ok_or("Error read csv")?;
        let has_speed = values.iter().any(|v| v.hash == "4bd5c4e0a9" && v.value > 10.0);
        if has_speed {
            Ok(
                InputValues {
                    converter: self,
                    values: values
                }
            )
        } else {Err("Двигатель не запускался".into())}
    }
}

impl InputValues {
    pub fn make_values_3(self, step_sec: Duration) -> OutputValues {
        let values = &self.values;
        let name_hash = &self.converter.name_hash;

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
        let mut lst: Vec<Vec<String>> = std::iter::repeat(vec![String::from("");name_hash.len()]).take(cnt as usize+1).collect();
        let fields: HashMap<_, _> = name_hash.iter().zip(0..).map(|((_,hash), i)| ( hash.to_owned(), i)).collect();
        dbg!(&fields);
    //     let step_ms = step_sec.as_millis() as i64;
        for v in values {
            let i = (v.date_time.timestamp_millis() - dt_start.timestamp_millis())/(step_ms as i64);
            if let Some(f) = fields.get(&v.hash) {
    //             dbg!(i, *f);
                lst[i as usize][*f as usize] = format!("{:.2}", v.value).replace(".", ",");
            }
        }

        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
        OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt,
                step_sec: step_sec.as_secs_f32(),
            },
            fields: fields,
            values: lst,
        }
    }
}

impl OutputValues {
    pub fn write_csv(self) -> crate::MyResult {
        let conv = &self.converter;
        let info = &self.info;
        let new_path = conv.output_path
            .join(format!("{}_filter_{}.csv", conv.file_name, info.step_sec));
        use std::fs::OpenOptions;
        let mut wrt = ::csv::WriterBuilder::new()
            .delimiter(b';')
    //         .from_writer(file);
            .from_path(new_path)?;
        wrt.write_record(&self.fields).unwrap();

        for s in self.values {
            if !s[0].is_empty() {
                wrt.write_record(&s)?;
            }
        }
        Ok(())
    }
}

pub fn filter_values_3(file_name: &str, step_sec: Duration, name_hash: Vec<(&str, &str)>) -> crate::MyResult {
    let cur_path = crate::get_file_path("csv/").join(file_name.to_owned()+".csv");

    let values = crate::csv::read_values(&cur_path).ok_or("Error read csv")?;
    let has_speed = values.iter().any(|v| v.hash == "4bd5c4e0a9" && v.value > 10.0);
    let speed_str = if has_speed {"+"} else {"-"};
    let new_path = crate::get_file_path("csv/").join(format!("{}{}_filter_{}.csv",speed_str, file_name,step_sec.as_secs_f32()));
    dbg!(&new_path);

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
    let mut lst: Vec<Vec<String>> = std::iter::repeat(vec![String::from("");name_hash.len()]).take(cnt as usize+1).collect();
    let fields: HashMap<_, _> = name_hash.iter().zip(0..).map(|((_,hash), i)| ( hash.to_owned(), i)).collect();
    dbg!(&fields);
//     let step_ms = step_sec.as_millis() as i64;
    for v in values {
        let i = (v.date_time.timestamp_millis() - dt_start.timestamp_millis())/(step_ms as i64);
        if let Some(f) = fields.get(&v.hash.as_str()) {
//             dbg!(i, *f);
            lst[i as usize][*f as usize] = format!("{:.2}", v.value).replace(".", ",");
        }
    }

    let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
    use std::fs::OpenOptions;
    let mut wrt = ::csv::WriterBuilder::new()
        .delimiter(b';')
//         .from_writer(file);
        .from_path(new_path)?;
    wrt.write_record(&fields).unwrap();

    for s in lst {
        if !s[0].is_empty() {
            wrt.write_record(&s)?;
        }
    }

    Ok(())
}

pub fn filter_values(file_name: &str, step_sec: Duration, name_hash: Vec<(&str, &str)>) -> crate::MyResult {
    let step_sec = step_sec.as_secs_f32();
    let cur_path = crate::get_file_path("csv/").join(file_name.to_owned()+".csv");
    
    let values = crate::csv::read_values(&cur_path).ok_or("Error read csv")?;
    let has_speed = values.iter().any(|v| v.hash == "4bd5c4e0a9" && v.value > 10.0);
    let speed_str = if has_speed {"+"} else {"-"};
    let new_path = crate::get_file_path("csv/").join(format!("{}{}_filter_{}.csv",speed_str, file_name,step_sec));
    dbg!(&new_path);
    
    let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
    let cnt = values.iter().filter(move |v| v.hash == "4bd5c4e0a9").count();
    dbg!(&dt_dlt, &cnt);
    let step_sec = (step_sec * 100.0) as i32;
    let dt_dlt = dt_dlt * 100;
    let stp = cnt as f32 / (dt_dlt /step_sec as i32).num_seconds() as f32;
    dbg!(&stp);
    let stp = stp.round() as usize;
    if stp == 0 {dbg!(&stp);return Err("Step = 0".into());}
    
//     let name_hash = vec![
//         ("dt", "2) МВ110-24.8АС/5/value"),
//         ("Температура ротора", "2) МВ110-24.8АС/5/value"),
//         ("Вибродатчик", "2) МВ110-24.8АС/7/value"),
//         ("Температура статора", "1) МВ210-101/1/value"),
//         ("Температура масла на выходе дв. М1 Низ", "1) МВ210-101/2/value"),
//         ("Температура подшипника дв. М1 верх", "1) МВ210-101/6/value"),
//     ];

    let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
    let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
        values.iter()
            .filter(move |v| v.hash == hash)
            .zip(0..cnt)
            .map(move |(v,i)| 
//             if name == "dt" {
//                 format!("{1};{0}", i/stp, 
//                 (v.date_time+crate::Duration::hours(3)).format("%H:%M:%S").to_string()
//                 )
//             } else {
                format!("{:.2}", v.value)
//             }
            ).step_by(stp)
    }).collect();
//     lst.insert(1, (1..cnt).filter(|v| true).map(|v| v).step_by(stp));
    
//     let dts: Vec<_> = 
//     dbg!(&lst);
    let lst : Vec<_> = MyZip::new(lst)
//     .take(10)
//     .map(|v| v.into_string())
    .collect();
//     dbg!(&lst);
    let cnt = lst.len();

    use std::fs::OpenOptions;
//     use csv::WriterBuilder;
//     let file = File::create(file_name)?;
//     let file = OpenOptions::new().open(new_path)?;
//     dbg!(&file);
    let mut wrt = ::csv::WriterBuilder::new()
//         .has_headers(true)
        .delimiter(b';')
//         .from_writer(file);
        .from_path(new_path)?;
    wrt.write_record(&fields).unwrap();
    
    for s in lst {
        wrt.write_record(&s)?;
    }
    
    println!("OK ({})\n", cnt);
    Ok(())
}

pub fn filter_values_2(file_name: &str, step_sec: u16) -> crate::MyResult {
    let cur_path = crate::get_file_path("csv/").join(file_name.to_owned()+".csv");
    let new_path = crate::get_file_path("csv/").join(format!("{}_filter_{}.csv",file_name,step_sec));
    
    let values = crate::csv::read_values(&cur_path).ok_or("Error read csv")?;
    let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
    dbg!(&new_path, &dt_dlt);
    
    let name_hash = vec![
        ("dt", "2) МВ110-24.8АС/5/value"),
        ("Температура ротора", "2) МВ110-24.8АС/5/value"),
        ("Вибродатчик", "2) МВ110-24.8АС/7/value"),
        ("Температура статора", "1) МВ210-101/1/value"),
        ("Температура масла на выходе дв. М1 Низ", "1) МВ210-101/2/value"),
        ("Температура подшипника дв. М1 верх", "1) МВ210-101/6/value"),
    ];
    
    use std::collections::BTreeMap;
//     let map_values = BTreeMap::new();
    
    let felds: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
    let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
        let dt_value: BTreeMap<_, _> = values.iter()
            .filter(move |v| v.hash == hash)
            .map(move |v| {
                let dt = (v.date_time+crate::Duration::hours(3)).format("%H:%M:%S").to_string();
                if name == "dt" { ( dt.clone(), dt) }
                else {(dt, format!("{:.1}", v.value) )}
            }).collect();
        let val: Vec<_> = dt_value.values().cloned().collect();
        val
    }).collect();
    
    let lst : Vec<_> = myzip(lst)
//         .take(5)
        .collect();
        
    let mut wrt = ::csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(new_path)?;
    for s in lst {
        wrt.write_record(&s)?;
    }
    
    Ok(())
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
fn myzip<T, U>(vec: Vec<T>) -> MyZip<T::IntoIter, U>
where
    T: IntoIterator<Item=U>
{
    MyZip::new(vec.into_iter()
        .map(|v| v.into_iter())
        .collect())
}
