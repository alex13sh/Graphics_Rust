pub use chrono::{SecondsFormat, Offset, FixedOffset, Duration};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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

pub fn date_time_to_string_name_short3(dt: &DateTime) -> String {
    date_time_to_string_name_short(&(*dt+Duration::hours(3)))
}
pub fn date_time_to_string_name_short(dt: &DateTime) -> String {
    (*dt).format("%Y_%m_%d-%H_%M_%S")
        .to_string()
}

pub fn date_time_to_string_name_hum(dt: &DateTime) -> String {
    (*dt+Duration::hours(3)).format("%d.%m.%Y-%H:%M:%S")
        .to_string()
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

pub(crate) fn float_to_str<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{:.2}", value);
    serializer.serialize_str(&s)
}

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
