use super::Value;
// use super::Device;

use super::init::{ValueError, SensorType, SensorAnalogType};
use super::init::ValueGroup as SensorInit;

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
struct GroupPin {
    name: String,
    pin: u8,
    values: Vec<Arc<Value>>,
    value: Arc<Value>,
}
struct GroupValue {
    name: String,
    values: Vec<Arc<Value>>,
}
enum ValueGroup {
    Sensor(Sensor),
    GroupPin(GroupPin),
    Group(GroupValue),
}

impl Sensor {
    pub fn new(s: SensorInit, values: Vec<Arc<Value>>, value: Arc<Value>) -> Self {
        match s {
        SensorInit::Sensor {name, pin, value_error, sensor_type, interval} => {
            Sensor {
                name: name,
                pin: pin,
                value_error: value_error,
                sensor_type: sensor_type,
                interval: interval,
                values: values,
                value: value
            }
        },
        SensorInit::GroupPin {name, pin, group_type:typ}=> {
            Sensor {
                name: name,
                pin: pin,
                values: values,
                value: value,
                interval: 1000,
                .. Sensor::default()
            }
        },
        _ => Sensor::default()
        }
    }
}

impl Into<f32> for Sensor {
    fn into(self) -> f32 {
//         self.value_float()
        0_f32
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
