#![allow(dead_code, unused_variables, unused_imports)]

pub use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
pub use chrono::{SecondsFormat, Offset, FixedOffset, Duration};

pub type DateTimeLocal = chrono::DateTime<chrono::Local>;
pub type DateLocal = chrono::Date<chrono::Local>;
pub type DateTimeFix = chrono::DateTime<chrono::FixedOffset>;
// type DateTimeMSK = chrono::DateTime<MSK>;
pub type DateTime = DateTimeFix;

pub fn date_time_now() -> DateTime {
    DateTime::from(chrono::Local::now())
//         .east(3*60*60)
}

pub fn date_time_to_string_name(dt: &DateTime) -> String {
    dt.format("%d_%m_%Y__%H_%M_%S_%.f")
        .to_string().replace("_.", "_")
}

pub fn date_time_to_string_name_short(dt: &DateTime) -> String {
    (*dt+Duration::hours(3)).format("%d_%m_%Y %H_%M_%S")
        .to_string()
}

pub mod structs;
pub mod json;
pub mod csv;

#[cfg(feature = "convert")]
pub mod convert;

// pub use json::*;
// pub use csv::*;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;

pub fn get_file_path(file_name: &str) -> PathBuf {
    let mut path: PathBuf = if let Some(project_dirs) =
        directories::ProjectDirs::from("rs", "modbus", "GraphicModbus")
    {
        project_dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or(PathBuf::new())
    };
//     path = std::path::Path::new("/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log/").into();
    path.push(file_name);
    path
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvertorParametr {
    pub address: String, //(u8, u8),
    pub value: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogValueRaw {
    #[serde(deserialize_with = "date_time_from_str")]
    #[serde(serialize_with = "date_time_to_str")]
    pub date_time: DateTimeFix,
    pub hash: String,
    pub value: f32,
}

impl LogValueRaw {
    pub fn new(hash: String, value: f32) -> Self {
//         dbg!(&hash, &value);
        LogValueRaw {
            date_time: date_time_now(),
            hash: hash,
            value: value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogValueHum {
    #[serde(deserialize_with = "date_time_from_str")]
    #[serde(serialize_with = "date_time_to_str")]
    pub date_time: DateTimeFix,
    pub device_id: u16,
    pub device_name: String,
    #[serde(rename = "value_name")]
    pub sensor_name: String,
//     pub value_name: String,
    pub value: f32,
}

use serde::{de, Deserializer, Serializer};
pub(crate) fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTimeFix, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s +" +0300";
    let dt = DateTimeFix::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f %z").map_err(de::Error::custom)?;
    Ok(dt-Duration::hours(3))
}

pub(crate) fn date_time_to_str<S>(dt: &DateTimeFix, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
//     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    let s = (*dt+Duration::hours(3))
    .format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
    serializer.serialize_str(&s)
}

enum LoggerType {
    Json {
        sessions: Vec<json::NewJsonLog>,
    },
    CSV {
        sessions: Vec<csv::SessionTime>,
    },
}

pub struct Logger {
    log_type: LoggerType,
    
}

impl Logger {
    pub fn open_json() -> Self {
        Logger {
            log_type: LoggerType::Json {
                sessions: Vec::new(),
            },
        }
    }
    pub fn open_csv() -> Self {
        let sessions_path = get_file_path("csv/session.csv");
        Logger {
            log_type: LoggerType::CSV {
                sessions: csv::read_session_full(&sessions_path)
                    .unwrap_or(Vec::new()),
            },
        }
    }
    
    pub fn get_last_values(&self) -> Option<&Vec<crate::LogValueRaw>> {
        match self.log_type {
        LoggerType::CSV {ref sessions} => sessions.last()?.values.as_ref(),
        LoggerType::Json {ref sessions} => Some(&sessions.last()?.values),
        _ => None,
        }
    }
    
    pub fn new_session(&mut self, values: &Vec<crate::LogValueRaw>) {
        match self.log_type {
        LoggerType::CSV {ref mut sessions} => {
            let start = values.first().unwrap().date_time;
            let finish = values.last().unwrap().date_time;
            let s = csv::SessionTime {
                start: start,
                finish: finish,
                file_name: Some(format!("value_{}.csv", date_time_to_string_name_short(&start))),
                values: Some(values.clone()),
            };
            sessions.push(s);
            csv::write_session(&get_file_path("csv/session.csv"), sessions.clone());
        }
//         LoggerType::Json {ref _sessions} => {},
        _ => {},
        }
    }
    
}

pub fn new_csv_raw(values: &Vec<crate::LogValueRaw>) {
    if values.len() < 2 {return;}
    let start = values.first().unwrap().date_time;
    
    let file_name = format!("value_{}.csv", date_time_to_string_name_short(&start));
    csv::write_values(&get_file_path("tables/csv/").join(file_name), values);
}

pub fn new_table_fields(values: Vec<crate::LogValueRaw>, step_sec: u16, name_hash: Vec<(&str, &str)>) -> Option<(structs::TableState, PathBuf)> {
    if values.is_empty() {return None;}
    use std::time::Duration;
    let start = values[0].date_time;
    let values = structs::Converter::output_file(crate::get_file_path("tables/excel/"),
        &format!("{}", date_time_to_string_name_short(&start)))
        .from_log_values(values)
        .fields(name_hash)
        .make_values_3(Duration::from_millis(100))
            .fill_empty()
            .shift_vibro()
            .insert_time_f32();
    let mut res = (
        values.get_state(),
        values.converter.as_ref().unwrap().get_output_file_path().with_extension("xlsx")
    );
    res.1 = values.write_excel().ok()?;
    Some(res)
}

pub fn open_log_state(file_name: &str) -> Option<(structs::TableState, PathBuf)> {
    use std::time::Duration;
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
    ];
    let values = structs::Converter::new(crate::get_file_path("tables/csv/"), crate::get_file_path("tables/excel/"))
        .read_file_opt(file_name, csv::read_values)?
        .fields(hashs)
        .make_values_3(Duration::from_millis(100))
            .fill_empty()
            .insert_time_f32();
    Some((
        values.get_state(),
        crate::get_file_path("tables/excel/").join(file_name).with_extension("xlsx")
    ))
}

fn hash_to_names(hash: &str) -> (u16, String, String) {
    match hash {
    "Температура статора дв. М2/value" => (1, "МВ210-101".into(), "Температура статора дв. М2".into()),
    "Температура верх подшипника дв. М2/value" => (1, "МВ210-101".into(), "Температура верх подшипника дв. М2".into()),
    "Температура нижн подшипника дв. М2/value" => (1, "МВ210-101".into(), "Температура нижн подшипника дв. М2".into()),
    "Температура статора двигатель М1/value" => (1, "МВ210-101".into(), "Температура статора двигатель М1".into()),
    "Температура масла на верхн. выходе дв. М1/value" => (1, "МВ210-101".into(), "Температура масла на верхн. выходе дв. М1".into()),
    "Температура масла на нижн. выходе дв. М1/value" => (1, "МВ210-101".into(), "Температура масла на нижн. выходе дв. М1".into()),
    "Температура масла на выходе маслостанции/value" => (1, "МВ210-101".into(), "Температура масла на выходе маслостанции".into()),
    "Давление масла на выходе маслостанции/value" => (2, "МВ110-24.8АС".into(), "Давление масла на выходе маслостанции".into()),
    "Давление воздуха компрессора/value" => (2, "МВ110-24.8АС".into(), "Давление воздуха компрессора".into()),
    "Разрежение воздуха в системе/value" => (2, "МВ110-24.8АС".into(), "Разрежение воздуха в системе".into()),
    "Температура ротора Пирометр дв. М1/value" => (2, "МВ110-24.8АС".into(), "Температура ротора Пирометр дв. М1".into()),
    "Температура ротора Пирометр дв. М2/value" => (2, "МВ110-24.8АС".into(), "Температура ротора Пирометр дв. М2".into()),
    "Виброскорость дв. М1/value" => (2, "МВ110-24.8АС".into(), "Виброскорость дв. М1".into()),
    "Виброскорость дв. М2/value" => (2, "МВ110-24.8АС".into(), "Виброскорость дв. М2".into()),
    
    "Битовая маска состояния выходов" => (3, "МК210-302".into(), "Битовая маска состояния выходов".into()),
    "Битовая маска состояния входов" => (3, "МК210-302".into(), "Битовая маска состояния входов".into()),
    "Клапан ШК1 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК1 открыт".into()),
    "Клапан ШК1 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК1 закрыт".into()),
    "Клапан ШК2 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК2 открыт".into()),
    "Клапан ШК2 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК2 закрыт".into()),
    "Клапан ШК3 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК3 открыт".into()),
    "Клапан ШК3 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК3 закрыт".into()),
    "Клапан ШК4 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК4 открыт".into()),
    "Клапан ШК4 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК4 закрыт".into()),
    "Клапан ШК5 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК5 открыт".into()),
    "Клапан ШК5 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК5 закрыт".into()),
    "Клапан ШК6 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК6 открыт".into()),
    "Клапан ШК6 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК6 закрыт".into()),
    "Двигатель насоса вакуума 1/write_bit" => (3, "МК210-302".into(), "Двигатель насоса вакуума 1".into()),
    "Двигатель насоса вакуума 2/write_bit" => (3, "МК210-302".into(), "Двигатель насоса вакуума 2".into()),
    
    "Битовая маска состояния выходов" => (4, "МУ210-410".into(), "Битовая маска состояния выходов".into()),
    "Двигатель подачи материала в камеру/Частота высокочастотного ШИМ" => (4, "МУ210-410".into(), "Двигатель подачи материала в камеру".into()),
    "Направление вращения двигателя ШД/write_bit" => (4, "МУ210-410".into(), "Направление вращения двигателя ШД".into()),
    "Двигатель маслостанции М4/write_bit" => (4, "МУ210-410".into(), "Двигатель маслостанции М4".into()),
    "Двигатель компрессора воздуха/write_bit" => (4, "МУ210-410".into(), "Двигатель компрессора воздуха".into()),
    "Клапан нижнего контейнера/write_bit" => (4, "МУ210-410".into(), "Клапан нижнего контейнера".into()),
    "Клапан подачи материала/write_bit" => (4, "МУ210-410".into(), "Клапан подачи материала".into()),
    "Клапан помольной камеры/write_bit" => (4, "МУ210-410".into(), "Клапан помольной камеры".into()),
    "Клапан напуска/write_bit" => (4, "МУ210-410".into(), "Клапан напуска".into()),
    "Клапан верхнего контейнера/write_bit" => (4, "МУ210-410".into(), "Клапан верхнего контейнера".into()),
    "Клапан насоса М5/write_bit" => (4, "МУ210-410".into(), "Клапан насоса М5".into()),
    
    "ac4e9ff84c" => (5, "Invertor".into(), "Наработка двигателя (мин)".into()),
    "b735f11d88" => (5, "Invertor".into(), "Наработка двигателя (дни)".into()),
    "4c12e17ba3" => (5, "Invertor".into(), "Заданная частота (F)".into()),
    "4bd5c4e0a9" => (5, "Invertor".into(), "Скорость двигателя".into()),
    "5146ba6795" => (5, "Invertor".into(), "Выходной ток (A)".into()),
    "Напряжение на шине DC" => (5, "Invertor".into(), "Напряжение на шине DC".into()),
    "5369886757" => (5, "Invertor".into(), "Выходное напряжение (E)".into()),
    "2206H" => (5, "Invertor".into(), "Индикация текущей выходной мощности (P)".into()),
    "2207H" => (5, "Invertor".into(), "Индикация рассчитанной (с PG) скорости".into()),
    "5b28faeb8d" => (5, "Invertor".into(), "Температура радиатора".into()),
    
    "6) Invertor/ac4e9ff84c" => (6, "Invertor".into(), "Наработка двигателя (мин)".into()),
    "6) Invertor/b735f11d88" => (6, "Invertor".into(), "Наработка двигателя (дни)".into()),
    "6) Invertor/4c12e17ba3" => (6, "Invertor".into(), "Заданная частота (F)".into()),
    "6) Invertor/4bd5c4e0a9" => (6, "Invertor".into(), "Скорость двигателя".into()),
    "6) Invertor/5146ba6795" => (6, "Invertor".into(), "Выходной ток (A)".into()),
    "6) Invertor/Напряжение на шине DC" => (6, "Invertor".into(), "Напряжение на шине DC".into()),
    "6) Invertor/5369886757" => (6, "Invertor".into(), "Выходное напряжение (E)".into()),
    "6) Invertor/2206H" => (6, "Invertor".into(), "Индикация текущей выходной мощности (P)".into()),
    "6) Invertor/2207H" => (6, "Invertor".into(), "Индикация рассчитанной (с PG) скорости".into()),
    "6) Invertor/5b28faeb8d" => (6, "Invertor".into(), "Температура радиатора".into()),
    
    _ => (0, "".into(), "".into()),
    }
}

impl From<LogValueRaw> for LogValueHum {
    fn from(value: LogValueRaw) -> Self {
        let names = hash_to_names(&value.hash);
        LogValueHum {
            date_time: value.date_time,
            value: value.value,
            device_id: names.0,
            device_name: names.1,
            sensor_name: names.2,
        }
    }
}
