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

struct StateInfo {
//     material_time: DateTimeRange,
    max_values: MaxValues,
    energy: Energy,
    material: Option<StateMaterial>,
}

/// Пиковые значения: мощности, тока и вибрации
#[derive(Default)]
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
        }
        }
    }
}

#[derive(Default)]
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
        }
        }
    }
}

struct StateMaterial {
    time: DateTimeRange,
    energy: Energy,
    watt_before: f32,
}

impl StateMaterial {
    fn start(dt: DateTime) -> Self {
        StateMaterial {
            time: DateTimeRange::start(dt.clone()),
            energy: Energy::start(dt),

            watt_before: 0.0,
        }
    }

    fn finish(self) {

    }

    fn apply_value(&mut self, value: &LogValueSimple) {
        self.energy.apply_value(value);
        self.time.update(value.date_time);
    }

    /// Подсчёт всей энергии за время подачи материала
    pub fn energy(&self) -> f32 {
        self.energy.energy()
    }

    /// Подсчёт полезной энергии за время подачи материала
    pub fn energy_delta(&self) -> f32 {
        self.energy.energy_delta(self.watt_before)
    }
}

impl StateInfo {
    fn apply_line(&mut self, line: &SimpleValuesLine) {

        for v in line.iter_values_date() {
            self.apply_value(v);
        }
    }

    fn apply_value(&mut self, value: LogValueSimple) {
        self.max_values.apply_value(&value);
        self.energy.apply_value(&value);

        if let Some(ref mut mat) = self.material {
            mat.apply_value(&value);
        }

        match value.value.as_ref() {
        ValueStr {sensor_name: "Клапан ШК2 открыт", value: bit} => {
            if bit == 1.0 && self.material.is_none() {
                self.material = Some(StateMaterial::start(value.date_time.clone()));
            } else if bit == 0.0 && self.material.is_some() {
                // self.material.finish();
            }
        }
        }
    }
}

mod utils {
    pub use crate::utils::{
        DateTimeFix as DateTime,
        date_time_now
    };

    pub type TimeInterval = f32;

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
