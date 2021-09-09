#![allow(dead_code, unused_variables, unused_imports)]
#![feature(decl_macro)]

use log::*;
use std::time::Duration;

type MyResult = Result<(), Box<dyn std::error::Error>>;

fn main_1() -> MyResult {
//     convert_json_old_new()?;
//     convert_json2csv()?;
//     test_read_csv_2()?;
//     convert_session()?;
    let names = [
        "value_23_04_2021__14_10_44_951678936",
        "value_27_04_2021__12_48_14_722166742",
        "value_27_04_2021__13_36_14_460047525",
        "value_27_04_2021__13_37_46_459645663",
        "value_27_04_2021__13_38_46_459439921",
        "value_27_04_2021__13_39_32_958736343",
        "value_27_04_2021__13_43_35_959273451",
        
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
//     calc_hz()
//     compare_vibro()
//     test_group_path()
    compare_vibro_month()
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

use std::path::Path;

fn open_file_speed_vibro(file_path: &PathBuf) -> Option<structs::OutputValues> {
    let hashs = vec![
        ("Скорость", "4bd5c4e0a9"),
        ("Вибродатчик", "Виброскорость дв. М1/value"),
    ];
    let values_log = csv::read_values(file_path)?;
    let values = structs::InputValues::from_log_values(values_log)
        .fields(hashs)
        .make_values_3(Duration::from_millis(100))
            .fill_empty()
            .shift_vibro()
        .conert_to_speed_values(15_000, 100);
    Some(values)
}
fn path_to_string(path: &PathBuf) -> Option<String> {
    let file_name = path.file_name()?
        .to_owned().into_string().ok()?;
    let file_name = file_name.strip_prefix("value_")?
        .strip_suffix(".csv")?.to_owned();
    Some(file_name)
}
fn compare_vibro() -> crate::MyResult {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    
    for f in get_file_list("tables/csv/").iter().rev().take(8) {
        let values = open_file_speed_vibro(&f).ok_or("Ошибка чтения файла")?;        
        map.insert(path_to_string(f).unwrap(), values);
    }
    
    let values = merge_value_by_speed(map).ok_or("merge_values")?;
    values.write_excel_lite(&get_file_path("tables/table_speed_vibro (1000).xlsx"))?;
    Ok(())
}

fn merge_value_by_speed(
        mut values: impl IntoIterator<Item = (String, structs::OutputValues)>
    ) -> Option<structs::OutputValues> {
    let mut values = values.into_iter();
    let mut res = values.next()?.1;
    for ((n,v), i) in values.zip(2..) {
        res.fields.insert(i, n);
        res.values.insert_column(i, 
            v.values.into_iter()
            .map(|row| row[1])
        );
    }
    Some(res)
}
fn merge_value_vibro_average<'a>(
        mut values: impl Iterator<Item = &'a structs::OutputValues>
    ) -> Option<structs::OutputValues> {
    let mut res = values.next()?.clone();
    let mut cnt = 1;
    for v in values {
        for i in 0..res.values.0.len() {
            res.values.0[i][1] += v.values.0[i][1];
        }
        cnt+=1;
    }
    for i in 0..res.values.0.len() {
        res.values.0[i][1] /= cnt as f32;
    }
    Some(res)
}

pub macro batch_iner_m($itr:ident, $days:expr) {
    if let Some(p) = $itr.next() {
        let mut arr = Vec::new();
        let dt = p.0;
        arr.push(p.1);
        while let Some(p2) = $itr.peek() {
            let dt2 = p2.0;
            if dt2 - dt > chrono::Duration::days($days as i64) {break;}
            let p2 = $itr.next().unwrap();
            arr.push(p2.1);
        }
        Some((dt, arr))
    } else {None}
}

fn test_group_path() -> crate::MyResult {
    use itertools::Itertools;
    let paths : Vec<_> = get_file_list("tables/csv/").into_iter()
        .map(|p| (DateTimeLocal::from(p.metadata().unwrap().modified().unwrap()), p))
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 1)
        })
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 7)
        })
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 30)
        })
        .map(|(dt, v)| (dt.date(), v))
        .collect();
    dbg!(paths);
    Ok(())
}

fn compare_vibro_month() -> crate::MyResult {
    use itertools::Itertools;
    let out_path = get_file_path("tables/table_speed_vibro/");
    let paths : Vec<_> = get_file_list("tables/csv/").into_iter()
        .map(|p| (DateTimeLocal::from(p.metadata().unwrap().modified().unwrap()), p))
        .map(|(dt, p)| (dt, (path_to_string(&p).unwrap(), open_file_speed_vibro(&p).unwrap())))
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 1)
        })
        .inspect(|(dt, v)| {
            let p = date_time_to_string_name_short(&dt.clone().into());
            let values = merge_value_by_speed(v.clone()).unwrap();
            values.write_excel_lite(&out_path.join(&format!("{} (d1).xlsx", p))).unwrap()
        })
        .map(|(dt, v)| {
            let itr = v.iter().map(|(f,v)| v);
            let v = merge_value_vibro_average(itr).unwrap();
            let p = date_time_to_string_name_short(&dt.clone().into());
            (dt, (p, v))
        })
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 7)
        })
        .inspect(|(dt, v)| {
            let p = date_time_to_string_name_short(&dt.clone().into());
            let values = merge_value_by_speed(v.clone()).unwrap();
            values.write_excel_lite(&out_path.join(&format!("{} (d7).xlsx", p))).unwrap()
        })
        .map(|(dt, v)| {
            let itr = v.iter().map(|(f,v)| v);
            let v = merge_value_vibro_average(itr).unwrap();
            let p = date_time_to_string_name_short(&dt.clone().into());
            (dt, (p, v))
        })
        .peekable()
        .batching(|path| {
            batch_iner_m!(path, 30)
        })
        .inspect(|(dt, v)| {
            let p = date_time_to_string_name_short(&dt.clone().into());
            let values = merge_value_by_speed(v.clone()).unwrap();
            values.write_excel_lite(&out_path.join(&format!("{} (d30).xlsx", p))).unwrap()
        })
        .map(|(dt, v)| (dt.date(), v))
        .collect();
    
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

