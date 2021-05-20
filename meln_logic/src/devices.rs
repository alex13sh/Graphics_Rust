use modbus::{Value, ModbusValues, ValueError};
use std::sync::Arc;

pub struct Dozator {
    hz_value: Arc<Value>,
//     hz: u32,
}

impl Dozator {
    pub fn new(values: ModbusValues) -> Option<Self> {
        Some(Self {
            hz_value: values.get("Двигатель подачи материала в камеру/Период низко частотного ШИМ")?.clone(),
//             hz: 0,
        })
    }
    pub async fn set_value(&mut self, finish_value: u32) {
        use tokio::time::sleep;
        use std::time::Duration;
        let start_value = self.hz_value.value();
        let dlt_value = (finish_value - start_value) as f32 / 10_f32;
        for i in 0..10 {
            let i = i as f32;
            self.hz_value.set_value(
                (start_value as f32 + dlt_value as f32 * i) as u32
            );
            sleep(Duration::from_millis(100)).await;
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
