// use super::Value;
use super::Sensor;
use super::Value;

use super::init::{DeviceType};
use super::init::Device as DeviceInit;

use std::sync::Arc;

#[derive(Debug)]
pub struct Device {
    name: String,
    sensors: Vec<Sensor>,
    values: Vec<Arc<Value>>,
    device_type: DeviceType
}

impl From<DeviceInit> for Device {
    fn from(d: DeviceInit) -> Device {
        Device {
            name: d.name,
            device_type: d.device_type,
            sensors: d.sensors.unwrap_or(Vec::new()).into_iter().map(|s| Sensor::from(s)).collect(),
            values: Vec::new(),
        }
    }
}

// impl DeviceType {
//     pub fn new_sensor(&self) -> Sensor { // TODO: Изменить тип сенсора
//         match *self {
//             Self::OwenAnalog => {
//                 Sensor::default()
//             },
//             Self::OwenDigitalIO {
//                 Sensor::default()
//             }
//         }
//     }
// }
