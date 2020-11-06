// use super::Value;
use super::Sensor;
use super::Value;

use super::init::{DeviceType};
use super::init::Device as DeviceInit;
use super::init::ValueGroup as SensorInit;

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
        let typ = &d.device_type;
        let sens = d.sensors.unwrap_or(Vec::new()).into_iter().map(|s| typ.new_sensor(s));
        Device {
            name: d.name,
            sensors: sens.collect(),
            device_type: d.device_type,
            values: Vec::new(),
        }
    }
}

impl DeviceType {
    pub fn new_sensor(&self, s: SensorInit) -> Sensor { // TODO: Изменить тип сенсора
        let values;
        let value;
        match *self {
        Self::OwenAnalog => {
            match s {
            SensorInit::Sensor{pin, ..} => {
                values = create_values_owen_analog(pin);
                value = values[0].clone();
                
            },
            _ => {
                values = Vec::new();
                value = Arc::new(Value::default())
            }
            }
        },
        Self::OwenDigitalIO => {
            values = create_values_owen_digital();
            value = Arc::new(Value::default())
        }
        }
        Sensor::new(s, values, value )
    }
    
}

fn create_values_owen_analog(pin: u8) -> Vec<Arc<Value>> {
    Vec::new()
}
fn create_values_owen_digital(pin: u8) -> Vec<Arc<Value>> {
    Vec::new()
}
