/// Анализ грфиков, и построение таблички с информацией
/// Табличка должна состоять из:
/// 1) Мощность холостого хода (значение до подачи материала) .(оба двигателя)
/// 2) Максимальная мощность, токи, вибрация. (оба двигателя)
/// 3) Время подачи материала
/// 4) Энергия потрачена на всю работу, и на подачу материала.
/// И ещё можно поля с формулы
/// 1) Масса материала - входной параметр.
/// 2) Удельная энергия на грамм и килограмм.

use utils::{
    DateTime,
    DateTimeRange,
};

use crate::value::{
    self,
    simple::{Value, ValueStr}, SimpleValuesLine,
    LogValueSimple,
};

use futures::{Stream, StreamExt};

#[derive(Debug, Default, Clone)]
// #[derive(derive_more::Add)]
pub struct StateInfo {
//     material_time: DateTimeRange,
    max_values: MaxValues,
    energy: Energy,
    material: StateMaterial,
    
}

/// Пиковые значения: мощности, тока и вибрации
#[derive(Debug, Default, Clone)]
#[derive(derive_more::Add)]
struct MaxValues {
    /// мощность
    power: f32,
    /// ток
    amper: f32,
    /// вибрация
    vibro: f32,
}

impl MaxValues {
    fn apply_value(&mut self, value: &LogValueSimple) {
        match value.value.as_ref() {
        ValueStr {sensor_name: "Виброскорость", value} => if self.vibro < value {
            self.vibro = value;
        }
        ValueStr {sensor_name: "Индикация текущей выходной мощности (P)", value} => if self.power < value {
            self.power = value;
        }
        ValueStr {sensor_name: "Выходной ток (A)", value} => if self.amper < value {
            self.amper = value;
        },
         _ => {}
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Energy {
    sum_watt: f32,
    sum_cnt: u32,
    time: DateTimeRange,
}

impl Energy {
    fn start(dt: DateTime) -> Self {
        Energy {
            time: DateTimeRange::start(dt),
            .. Default::default()
        }
    }

    /// Подсчёт всей энергии за время работы в Ватт*ч
    pub fn energy(&self) -> f32 {
        self.sum_watt / self.sum_cnt as f32
        * self.time.interval() / 60.0 / 60.0
    }
    /// Подсчёт разницы энергии за время работы в Ватт*ч
    pub fn energy_delta(&self, watt_low: f32) -> f32 {
        (self.sum_watt / self.sum_cnt as f32 - watt_low)
        * self.time.interval() / 60.0 / 60.0
    }

    fn apply_value(&mut self, log_value: &LogValueSimple) {
        match log_value.value.as_ref() {
        ValueStr {sensor_name: "Индикация текущей выходной мощности (P)", value} => {
            self.sum_watt += value;
            self.sum_cnt += 1;
            self.time.update(log_value.date_time.clone());
        },
        _ => {}
        }
    }
}

#[derive(Debug, Clone)]
enum StateMaterial {
    Before {
        watt_before: f32,
    },
    Start {
        time: DateTimeRange,
        energy: Energy,
        watt_before: f32,
        max_values: MaxValues,
    },
    Finish (StateMaterialFinish)
}

#[derive(Debug, Clone)]
pub struct StateMaterialFinish {
    time_interval: f32,
    time_range: DateTimeRange,

    energy_full: f32,
    energy_delta: f32,

    watt_before: f32,
}

impl Default for StateMaterial {
    fn default() -> Self {
        StateMaterial::empty()
    }
}

impl StateMaterial {
    fn empty() -> Self {
        StateMaterial::Before { watt_before: 0.0 }
    }

    fn start(self, dt: DateTime) -> Self {
        if let StateMaterial::Before { watt_before } = self {
            StateMaterial::Start {
                time: DateTimeRange::start(dt.clone()),
                energy: Energy::start(dt),
                watt_before,
                max_values: MaxValues::default(),
            }
        } else {
            self
        }
    }

    fn finish(self) -> Self {
        if let StateMaterial::Start { energy, watt_before, time, .. } = self {
            StateMaterial::Finish (StateMaterialFinish {
                energy_full: energy.energy(),
                energy_delta: energy.energy_delta(watt_before),
                time_interval: time.interval(),
                time_range: time,

                watt_before,
            })
        } else {
            self
        }
    }

    fn apply_value(&mut self, value: &LogValueSimple) {
        match self {
        Self::Before {watt_before} => {
            match value.value.as_ref() {
                ValueStr {sensor_name: "Клапан подачи материала открыт", value: bit} => if bit == 1.0 {
                    if *watt_before >= 1.0 {
                        *self = self.clone().start(value.date_time.clone()); 
                    }
                },
                ValueStr {sensor_name: "Индикация текущей выходной мощности (P)", value} => {
                    *watt_before = value;
                },
                _ => {}
            }
        }
        Self::Start {energy, time, watt_before, max_values} => {
            energy.apply_value(value);
            max_values.apply_value(value);
            time.update(value.date_time);

            match value.value.as_ref() {
                ValueStr {sensor_name: "Клапан подачи материала открыт", value: bit} => if bit == 0.0 {
                    *self = self.clone().finish();
                },
                ValueStr {sensor_name: "Индикация текущей выходной мощности (P)", value} => {
                    let watt_delta = max_values.power - *watt_before;
                    let watt_cur_proc = (value - *watt_before) / watt_delta;
                    if watt_delta > 1.0 && watt_cur_proc < 0.1 {
                        *self = self.clone().finish();
                    }
                },
                _ => {}
            }
        },
        Self::Finish {..} => {}
        }
    }

    /// Подсчёт всей энергии за время подачи материала
    pub fn energy(&self) -> f32 {
        match self {
            Self::Before {..} => {0.0}
            Self::Start {energy, ..} => energy.energy(),
            Self::Finish (stat) => {stat.energy_full}
        }
    }

    /// Подсчёт полезной энергии за время подачи материала
    pub fn energy_delta(&self) -> f32 {
        match self {
            Self::Before {..} => {0.0}
            Self::Start {energy, watt_before, ..} => energy.energy_delta(*watt_before),
            Self::Finish (stat) => {stat.energy_delta}
        }
    }

    pub fn get_stat(&self) -> Option<StateMaterialFinish> {
        if let Self::Finish(stat) = self {
            Some(stat.clone())
        } else {
            None
        }
    }
}

impl StateInfo {
    fn apply_line(&mut self, line: &SimpleValuesLine) {
        // dbg!(&line.date_time);
        for v in line.iter_values_date() {
            self.apply_value(v);
        }
    }

    fn apply_value(&mut self, value: LogValueSimple) {
        self.max_values.apply_value(&value);
        self.energy.apply_value(&value);
        self.material.apply_value(&value);
    }
}

mod utils {
    pub use crate::utils::{
        DateTimeFix as DateTime,
        date_time_now
    };

    pub type TimeInterval = f32;

    #[derive(Debug, Clone)]
    #[derive(PartialEq)]
    pub enum DateTimeRange {
        None,
        Start(DateTime),
        Range(DateTime, DateTime),
    }

    impl Default for DateTimeRange {
        fn default() -> Self {
            DateTimeRange::None
        }
    }

    impl DateTimeRange {
        pub fn empty() -> Self {
            DateTimeRange::None
        }
        pub fn start(dt: DateTime) -> Self {
            DateTimeRange::Start(dt)
        }
        pub fn update(&mut self, dt: DateTime) {
            *self = match *self {
            DateTimeRange::None => DateTimeRange::Start(dt),
            DateTimeRange::Start(start) => DateTimeRange::Range(start, dt),
            DateTimeRange::Range(start, _) => DateTimeRange::Range(start, dt),
            };
        }
        pub fn interval(&self) -> TimeInterval {
            if let Self::Range(start, finish) = self {
                // start.signed_duration_since(finish.clone()).num_seconds();
                (finish.timestamp_millis() - start.timestamp_millis()) as f32 / 1000.0
            } else {
                0.0
            }
        }

        fn get_start(&self) -> Option<&DateTime> {
            match self {
            Self::None => None,
            Self::Start(dt) => Some(dt),
            Self::Range(dt, _) => Some(dt),
            }
        }
        fn get_range(&self) -> (Option<&DateTime>, Option<&DateTime>) {
            match self {
            Self::None => (None, None),
            Self::Start(dt) => (Some(dt), None),
            Self::Range(dt_1, dt_2) => (Some(dt_1), Some(dt_2)),
            }
        }
        fn into_range(self) -> (Option<DateTime>, Option<DateTime>) {
            match self {
            Self::None => (None, None),
            Self::Start(dt) => (Some(dt), None),
            Self::Range(dt_1, dt_2) => (Some(dt_1), Some(dt_2)),
            }
        }
        fn from_range(range: (Option<DateTime>, Option<DateTime>)) -> Self {
            match range {
            (None, None) => Self::None,
            (Some(dt), None) => Self::Start(dt),
            (Some(dt_1), Some(dt_2)) => Self::Range(dt_1, dt_2),
            _ => Self::None,
            }
        }
    }

    impl std::ops::Add for DateTimeRange {
        type Output = Self;

        fn add(self, rhs: Self) -> Self {
//             use Self::*;
//             match (self, rhs) {
//             (None, None) => None,
//             (Start(dt), None) | (None, Start(dt)) => Start(dt),
//             (Range(dt_1, dt_2), None) | (None, Range(dt_1, dt_2)) => Range(dt_1, dt_2),
//             (
//             }
            use std::cmp::{min, max};
            let range_1 = self.into_range();
            let range_2 = rhs.into_range();
            dbg!(&range_1);
            dbg!(&range_2);
            let range = (
                min(range_1.0, range_2.0),
                max(range_1.1, range_2.1),
            );
            dbg!(&range);
            Self::from_range(range)
        }
    }
}

pub fn calc(vin: impl Stream<Item=SimpleValuesLine>) -> impl Stream<Item=StateInfo> {
    let mut stat = StateInfo::default();

    vin.map(move |line| {
        stat.apply_line(&line);

        stat.clone()
    })
}

#[test]
fn test_convert_csv_raw_to_excel() {
    use crate::convert::{stream::*, iterator::*};
    use futures::future::join;

    let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw/";
    let file_names = [
        "2022_04_27-17_44_35",

        // "2022_04_18-16_38_06",
        // "2022_04_12-18_53_41",
        // "2022_04_12-18_50_01",
        // "2022_04_12-18_47_37",
        // "2022_04_12-18_46_53",
        // "2022_04_12-18_42_52",

        "2022_04_08-17_37_59",
        // "2022_04_08-17_32_20",

        "2022_03_29-13_58_12",
    ];

    for name in file_names {
        if let Some(values) =  crate::files::csv::read_values(&format!("{}{}.csv", dir, name)) {
            dbg!(name);
            let values = fullvalue_to_elk(values);
            let lines = values_to_line(futures::stream::iter(values));
            // let lines = values_line_to_simple(lines);
            let lines = super::filter_half_low(lines);
            let stat = 
                calc(lines)
                    .fold(None, |_, s| async{Some(s)});
            let stat = futures::executor::block_on(stat);
            dbg!(&stat);
            println!("Energy Low: {}", stat.unwrap().energy.energy());
            // assert!(false);
        }

        if let Some(values) =  crate::files::csv::read_values(&format!("{}{}.csv", dir, name)) {
            dbg!(name);
            let values = fullvalue_to_elk(values);
            let lines = values_to_line(futures::stream::iter(values));
            // let lines = values_line_to_simple(lines);
            let lines = super::filter_half_top(lines);
            let stat = 
                calc(lines)
                    .fold(None, |_, s| async{Some(s)});
            let stat = futures::executor::block_on(stat);
            dbg!(&stat);
            println!("Energy Top: {}", stat.unwrap().energy.energy());
            // assert!(false);
        }
    }
}

#[test]
fn test_date_range_sum() {
    // "1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z"
    // FixedOffset::east(0).ymd(1983, 4, 13).and_hms_milli(12, 9, 14, 274)
    pub use chrono::{SecondsFormat, Offset, FixedOffset, Duration};
    pub type DateTimeFix = chrono::DateTime<chrono::FixedOffset>;
    let dt_from_str = |s: &str| {
        let s = s.to_string() +" +0300";
        let dt = DateTimeFix::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f %z").ok()?;
        Some(dt-Duration::hours(3))
    };
    let dt_1 = dt_from_str("2022-04-27T17:44:35.966").unwrap();
    let dt_2 = dt_from_str("2022-04-27T17:44:38.355").unwrap();
    let dt_3 = dt_from_str("2022-04-27T17:44:39.258").unwrap();
    let dt_4 = dt_from_str("2022-04-27T17:44:40.207").unwrap();
    let mut dt_range_1 = DateTimeRange::start(dt_1);
    dt_range_1.update(dt_3);
    let mut dt_range_2 = DateTimeRange::start(dt_2);
    dt_range_2.update(dt_4);

    let dt_range_12 = dt_range_1 + dt_range_2;
    let mut dt_range_3 = DateTimeRange::start(dt_1);
    dt_range_3.update(dt_4);
    assert_eq!(dt_range_12, dt_range_3);

    let dt_range_1 = DateTimeRange::start(dt_1);
    let mut dt_range_2 = DateTimeRange::start(dt_2);
    dt_range_2.update(dt_4);
    let dt_range_12 = dt_range_1 + dt_range_2;
    assert_eq!(dt_range_12, dt_range_3);

    let dt_range_1 = DateTimeRange::start(dt_2);
    let mut dt_range_2 = DateTimeRange::start(dt_1);
    dt_range_2.update(dt_4);
    let dt_range_12 = dt_range_1 + dt_range_2;
    assert_eq!(dt_range_12, dt_range_3);

    let dt_range_1 = DateTimeRange::empty();
    let mut dt_range_2 = DateTimeRange::start(dt_1);
    dt_range_2.update(dt_4);
    let dt_range_12 = dt_range_1 + dt_range_2;
    assert_eq!(dt_range_12, dt_range_3);
}

// impl std::ops::Add for MaxValues {
//     fn add(self, rhs: Self) -> Self {
//         Self {
//             power:
//         }
//     }
// }
