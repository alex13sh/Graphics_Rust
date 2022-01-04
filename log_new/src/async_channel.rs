// pub use async_broadcast::broadcast;

pub struct Sender<T>(async_broadcast::Sender<T>);
use futures::{Stream, StreamExt};

impl <T: Clone> Sender<T> {
    pub async fn send(&self, m: T) {
        let res = self.0.broadcast(m).await;
        res.unwrap();
    }
}

impl <T> From<async_broadcast::Sender<T>> for Sender<T> {
    fn from(f: async_broadcast::Sender<T>) -> Self {
        Self(f)
    }
}

pub fn broadcast<T>(cap: usize) -> (Sender<T>, async_broadcast::Receiver<T>)
{
    let (s, r) = async_broadcast::broadcast(cap);
    (Sender::from(s), r)
}

#[derive(Clone, Default)]
pub struct LogState {
    pub cnt: usize,
    date_time: Option<crate::utils::DateTimeFix>,
    pub time_all: f32,
    pub time_acel: f32,
    pub time_work: f32,
    
    pub hz_max: u32, // ValueHZ
    pub vibro_max: f32,
    pub hz_vibro: u32, // Зона вибрации
    pub tok_max: u32,
}

impl LogState {
    fn apply_line(&mut self, prev_line: Option<&crate::value::ElkValuesLine>, line: &crate::value::ElkValuesLine) {
        self.cnt += 1;
        
        if let Some(prev_line) = prev_line {
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
    fn apply_value(&mut self, value: crate::value::LogValueHum) {
        use crate::value::elk::Value;
        let dt = value.date_time;
        let sdt = self.date_time.as_ref().unwrap();
        match value.value.get_sensor_value() {
        ("Вибродатчик", value) => self.vibro_max = self.vibro_max.max(value),
        ("Скорость", value) => if self.hz_max <= value as u32 {
            self.hz_max = value as u32;
            self.time_acel = (dt.timestamp_millis() - sdt.timestamp_millis()) as f32 / 1000.0;
        }
        ("Ток", value) => self.tok_max = self.tok_max.max(value as u32),
        
        _ => {}
        }
    }
    
    fn apply_dlt_value(&mut self, prev_value: crate::value::LogValueHum, value: crate::value::LogValueHum) {
        
    }
}

pub fn calc_log_state(vin: impl Stream<Item=crate::value::ElkValuesLine>) -> impl Stream<Item=LogState> {
    let mut stat = LogState::default();
    let mut prev_line = None;
    vin.map(move |line| {
        stat.apply_line(prev_line.as_ref(), &line);
        prev_line = Some(line);
        stat.clone()
    })
}

