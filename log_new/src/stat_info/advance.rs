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
pub struct StateInfo {
//     material_time: DateTimeRange,
    max_values: MaxValues,
    energy: Energy,
    material: StateMaterial,
    
}

/// Пиковые значения: мощности, тока и вибрации
#[derive(Debug, Default, Clone)]
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
