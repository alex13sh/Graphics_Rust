#![allow(dead_code, unused_variables, unused_imports)]

use log::*;

type MyResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> MyResult {
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
    ];
    convert::filter_values(file_name, 1, hashs)?;
    Ok(())
}

fn filter_values_2(file_name: &str) -> crate::MyResult {
    let hashs = vec![
        ("Скорость", "4bd5c4e0a9"),
        ("Ток", "5146ba6795"),
        ("Вибродатчик", "OwenAnalog/3/value"),
        ("Температура ротора", "OwenAnalog/1/value"),
        ("Температура статора", "OwenAnalog/4/value"),
        ("Температура масла на выходе дв. М1 Низ", "OwenAnalog/6/value"),
        ("Температура подшипника дв. М1 верх", "OwenAnalog/5/value"),
    ];
    convert::filter_values(file_name, 1, hashs)?;
    Ok(())
}

// За 11 секунд и 30-40 мб озу
fn test_speed() -> MyResult {
    let paths = vec![
        "values_27_08_2020__13_08_30_042.json",
        "values_07_09_2020__13_02_37_096.json",
        "values_25_08_2020__13_41_06_111.json",
        "values_26_08_2020__16_26_04_840.json",
        "values_07_09_2020__16_13_35_221.json",
        "values_28_08_2020__16_57_26_959.json",
        "values_08_09_2020__14_28_27_576.json",
        "values_08_09_2020__14_28_33_906.json",
        "values_10_09_2020__15_36_13_274.json",
        "values_28_08_2020__17_06_20_523.json",
        "values_21_08_2020__17_31_00_188.json",
        "values_26_08_2020__15_48_12_214.json",
        "values_26_08_2020__16_05_51_804.json",
        "values_25_08_2020__15_15_21_933.json",
        "values_24_08_2020__19_19_10_684.json",
        "values_24_08_2020__19_03_16_045.json",
        "values_24_08_2020__18_31_00_766.json",
    ];
    for _ in 1..10 {
        for path in &paths {
            json::convert::convert_log_file(path, "Log/", "test_log")?;
        }
    }
    Ok(())
}

fn json_get_all_hash() -> MyResult {
    let js = json::open_json_file("values_14_09_2020__13_24_19_668.json");
    let hashs = js.get_all_hash();
    dbg!(hashs.len(), hashs);
    Ok(())
}

fn read_csv() {
    csv::test_read_csv_1("./log/sessions_1.csv")
        .unwrap();
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

// <<<<<<< Updated upstream

// fn convert_session() -> MyResult {
//     let session_path_1 = get_file_path("log/sessions_1.csv");
//     let session_path_2 = get_file_path("csv/sessions.csv");
//     
//     let sessions = csv::read_session(&session_path_1).ok_or("")?;
//     let sessions: Vec<_> = sessions.into_iter()
//         .map(|mut s| {
//             s.set_file_name(format!("value_{}.csv", date_time_to_string_name(&start)));
//             s
//         })
//         .collect();
//     csv::write_session(&session_path_2, sessions)?;
//     Ok(())
// }

fn get_file_list(dir: impl Into<PathBuf>) -> Vec<PathBuf> {
    let dir = dir.into();
    let dir_str = dir.to_str().unwrap();
    let path = get_file_path(dir_str);
    let paths = std::fs::read_dir(path).unwrap();
//     dbg!(paths);
    paths.filter_map(|res| res.ok())
    .map(|dir| dir.path())
    .filter(|path| 
        if let Some(ext) = path.extension() {
            ext == "json"
        } else {false}
    ).collect()
}

fn test_read_csv_2() -> MyResult {
    let tmp_path = get_file_path("csv/");
    
    let values = csv::read_values(&tmp_path.join("values_07_09_2020__16_42_09_399.csv")).ok_or("")?;
    dbg!(values);
    Ok(())
}
