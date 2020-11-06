use super::Value;
// use super::Device;

use super::init::{ValueError, SensorType, SensorAnalogType};
use super::init::Sensor as SensorInit;

use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Sensor {
    name: String,
    pin: u8,
    interval: u16,
    values: Vec<Arc<Value>>,
    value: Arc<Value>,
//     pub range: std::Range, 
    value_error: ValueError,
    sensor_type: SensorType,
}

impl From<SensorInit> for Sensor {
    fn from(s: SensorInit) -> Sensor {
        Sensor {
            name: s.name,
            pin: s.pin,
            value_error: s.value_error.unwrap_or(Default::default()),
            sensor_type: s.sensor_type,
            interval: s.interval.unwrap_or(1000),
            .. Sensor::default()
        }
    }
}

impl SensorType {
    fn value_float (&self) -> f32 {
        match *self {
        Self::Davl(_) => {
            // v = pow(10, v*10-5.5);
            0_f32
        },
        _ => 0_f32
        }
    }
    
    pub fn get_analog_type(&self) -> Option<&SensorAnalogType> {
        match self {
        Self::Analog(ref typ) => Some(typ),
        Self::Davl(_) => Some(&SensorAnalogType::Volt_1),
        _ => None
        }
    }
}
