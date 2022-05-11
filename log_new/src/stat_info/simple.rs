pub use super::{
    filter_half_top,
    filter_half_low,
};

use crate::value::{
    self,
    simple::Value, SimpleValuesLine,
    LogValueSimple,
};

use std::collections::HashMap;
use futures::{Stream, StreamExt};

#[derive(Clone, Default, Debug)]
pub struct LogState {
    pub cnt: usize,
    pub date_time: Option<crate::utils::DateTimeFix>,
    pub time_all: f32,
    pub time_acel: f32,
    pub time_work: f32,

    pub hz_max: u32, // ValueHZ
    pub vibro_max: f32,
    pub hz_vibro: u32, // Зона вибрации
    pub tok_max: u32,
    pub watt_max: f32,

    pub temps: HashMap<String, (f32, f32)>,
}

impl LogState {
    fn apply_line(&mut self, prev_line: Option<&SimpleValuesLine>, line: &SimpleValuesLine) {
        self.cnt += 1;

        if let Some(prev_line) = prev_line {
            self.time_all = (line.date_time.timestamp_millis() - self.date_time.unwrap().timestamp_millis()) as f32 / 1000.0;
            for (p, v) in prev_line.iter_values_date().zip(line.iter_values_date()) {
                self.apply_value(v.clone());
                self.apply_dlt_value(p, v);
            }
        } else {
            self.date_time = Some(line.date_time.clone());
            for v in line.iter_values_date() {
                self.apply_value(v);
            }
        }
    }
    fn apply_value(&mut self, value: LogValueSimple) {
        use value::simple::{Value, ValueStr};
        let dt = value.date_time;
        let sdt = self.date_time.as_ref().unwrap();
        match value.value.as_ref() {
        ValueStr {sensor_name: "Виброскорость", value} => if self.vibro_max < value {
            self.vibro_max = value;
            self.hz_vibro = self.hz_max; // При разгоне, текущая скорость = максимальная скорость
        }
        ValueStr {sensor_name: "Скорость двигателя", value} => if self.hz_max < value as u32 {
            self.hz_max = value as u32;
            self.time_acel = (dt.timestamp_millis() - sdt.timestamp_millis()) as f32 / 1000.0;
            // self.time_acel = self.time_all;
        }
        ValueStr {sensor_name: "Выходной ток (A)", value} => self.tok_max = self.tok_max.max(value as u32),
        ValueStr {sensor_name: "Индикация текущей выходной мощности (P)", value} => self.watt_max = self.watt_max.max(value as f32),
        ValueStr {sensor_name: sensor, value} if sensor.starts_with("Температура") => {
            if let Some(ref mut temp) = self.temps.get_mut(sensor) {
                temp.1 = value;
            } else {
                self.temps.insert(sensor.into(), (value, value));
            }
        }
        _ => {}
        }
    }

    fn apply_dlt_value(&mut self, _prev_value: LogValueSimple, _value: LogValueSimple) {

    }
}

pub fn calc(vin: impl Stream<Item=SimpleValuesLine>) -> impl Stream<Item=LogState> {
    let mut stat = LogState::default();
    let mut prev_line = None;
    vin.map(move |line| {
        stat.apply_line(prev_line.as_ref(), &line);
        prev_line = Some(line);
        stat.clone()
    })
}


