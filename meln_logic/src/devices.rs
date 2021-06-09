use modbus::{Value, ModbusValues, ValueError};
use std::sync::Arc;

use futures_core::stream::Stream;
use async_stream::stream;

#[derive(Clone)]
pub struct Dozator {
    hz_value: Arc<Value>,
//     hz: u32,
}

impl Dozator {
    pub fn new(values: ModbusValues) -> Option<Self> {
        Some(Self {
            hz_value: values.get("Двигатель подачи материала в камеру/Частота высокочастотного ШИМ")?.clone(),
//             hz: 0,
        })
    }
    pub fn set_value(&self, finish_value: u32) -> impl Stream<Item = u32>  {
        let hz_value = self.hz_value.clone();
        stream! {
        use tokio::time::sleep;
        use std::time::Duration;
        let steps = 20;
        let step_ms = 5_000/steps;
        let start_value = hz_value.value();
        let dlt_value = (finish_value as i32 - start_value as i32) as f32 / steps as f32;
        for i in 0..steps {
            let i = i as f32;
            let v = (start_value as f32 + dlt_value as f32 * i) as u32;
            hz_value.set_value(v);
            yield v;
            sleep(Duration::from_millis(step_ms)).await;
        }
        }
    }
    pub fn get_value(&self) -> u32 {
        self.hz_value.value()
    }
    pub fn get_value_f32(&self) -> f32 {
        use std::convert::TryFrom;
        TryFrom::try_from(self.hz_value.as_ref()).unwrap()
    }
}
