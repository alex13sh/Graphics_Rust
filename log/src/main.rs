#![allow(dead_code, unused_variables, unused_imports)]

use log::*;
use std::time::Duration;

type MyResult = Result<(), Box<dyn std::error::Error>>;

fn main_1() -> MyResult {
//     convert_json_old_new()?;
//     convert_json2csv()?;
//     test_read_csv_2()?;
//     convert_session()?;
    let names = [
        "value_15_10_2021 11_39_26",
        
    ];
    for name in &names {
        if let Err(txt) = filter_values(name) {
            println!("Error: {:?}", txt);
        }
    }
//     convert::filter_values_2("value_18_03_2021__13_18_44_814534674", 1)?;
//     find_1("value_18_03_2021__13_18_44_814534674");
    
    Ok(())
}

fn main() -> MyResult {
//     main_1()
//     calc_hz()
    print_file_state()
}

fn filter_values(file_name: &str) -> crate::MyResult {
    let hashs = vec![
        ("Скорость", "4bd5c4e0a9"),
        ("Ток", "5146ba6795"),
        ("Напряжение", "5369886757"),
        ("Вибродатчик", "Виброскорость дв. М1/value"),
        ("Температура ротора", "Температура ротора Пирометр дв. М1/value"),
        ("Температура статора дв. М1", "Температура статора двигатель М1/value"),
        ("Температура масла на верхн. выходе дв. М1", "Температура масла на верхн. выходе дв. М1/value"),
        ("Температура масла на нижн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1/value"),
        ("Давление масла на выходе маслостанции", "Давление масла на выходе маслостанции/value"),
        ("Разрежение воздуха в системе", "Разрежение воздуха в системе/value"),
    ];
    structs::Converter::new(crate::get_file_path("tables/csv/"), crate::get_file_path("tables/excel/"))
        .read_file_opt(file_name, csv::read_values).ok_or("Ошибка чтения файла")?
        .fields(hashs)
        .make_values_3(Duration::from_millis(100))
            .fill_empty()
            .shift_vibro()
            .convert_davl()
            .insert_time_f32()
        .write_excel()?;
    Ok(())
}

// Подсчёт времени работы на частотах.
fn calc_hz() -> crate::MyResult {
    let hashs = vec![
        ("Скорость", "4bd5c4e0a9"),
    ];
    let mut all_time_work_hz = vec![0_f32; 24];
    for f in get_file_list("tables/csv/")/*.iter().take(2)*/ {
        let values_log = csv::read_values(&f).ok_or("Ошибка чтения файла")?;
        let stat = structs::InputValues::from_log_values(values_log)
            .fields(hashs.clone())
            .make_values_3(Duration::from_millis(100))
                .fill_empty()
                .insert_time_f32()
            .get_state_build()
                .time_work_hz()
                .fields;
        let time_work_hz = stat.time_work_hz.clone().unwrap();
        for (t, hz) in time_work_hz.into_iter().zip(0..) {
            all_time_work_hz[hz] += t;
        }
    }
    dbg!(all_time_work_hz);
    Ok(())
}

fn print_file_state() -> crate::MyResult {
    let hashs = vec![
        ("Скорость", "4bd5c4e0a9"),
        ("Ток", "5146ba6795"),
        ("Вибродатчик", "Виброскорость дв. М1/value"),
        ("Давление масла на выходе маслостанции", "Давление масла на выходе маслостанции/value"),
        ("Разрежение воздуха в системе", "Разрежение воздуха в системе/value"),
        ("Температура ротора", "Температура ротора Пирометр дв. М1/value"),
    ];

    for f in get_file_list("tables/csv/").iter().rev().take(10) {
        let values_log = csv::read_values(&f).ok_or("Ошибка чтения файла")?;
        let stat = structs::InputValues::from_log_values(values_log)
            .fields(hashs.clone())
            .make_values_3(Duration::from_millis(100))
                .fill_empty()
                .insert_time_f32()
            .get_state();
        dbg!(f, stat);
    }
    Ok(())
}

fn convert_json_old_new() -> MyResult {
    use json::convert::*;
    
    let tmp_path = get_file_path("tmp/");
    let paths = get_file_list("log");
    let names = paths.iter()
        .filter_map(|path| path.file_name())
        .filter(|name| !tmp_path.join(name).exists());

    for name in names {
        convert_log_file(name.to_str().unwrap(), "log/", "tmp/")?;
    }
    
    Ok(())
}

fn convert_json2csv() -> MyResult {
    use convert::*;
    
    let tmp_path = get_file_path("csv/");
    let paths = get_file_list("tmp");
    let names = paths.iter()
        .filter_map(|path| path.file_stem())
        .filter(|name| !tmp_path.join(name).with_extension("csv").exists());
        
    for name in names {
        convert::json2csv(name.to_str().unwrap(), "tmp/", "csv/")?;
    }
    
    Ok(())
}


fn get_file_list(dir: &str) -> Vec<PathBuf> {
    let path = get_file_path(dir);
    let paths = std::fs::read_dir(path).unwrap();
//     dbg!(paths);
    let mut res: Vec<_> = paths.filter_map(|res| res.ok())
    .map(|dir| dir.path())
    .filter(|path| 
        if let Some(ext) = path.extension() {
            ext == "csv"
        } else {false}
    ).collect();
    res.sort_by_key(|p| p.metadata().unwrap().modified().unwrap());
    res
}

pub fn get_list_log(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut v = if dir.is_dir() {
        std::fs::read_dir(dir)?.into_iter()
            .filter_map(|e| Some(e.ok()?.path()))
            .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("csv") )
            .collect()
    } else {Vec::new()};
    v.sort_by_key(|p| p.metadata().unwrap().modified().unwrap());
    Ok(v)
}

