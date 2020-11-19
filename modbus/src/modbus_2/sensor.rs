#![allow(dead_code)]

use super::{Value, ModbusValues};
// use super::Device;

use super::init::{ValueError, SensorType, SensorAnalogType};
use super::init::ValueGroup as SensorInit;

use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Sensor {
    name: String,
    pin: u8,
    interval: u16,
    values: ModbusValues,
    value: Option<Arc<Value>>,
//     pub range: std::Range, 
    value_error: ValueError,
    sensor_type: SensorType,
}
struct GroupPin {
    name: String,
    pin: u8,
    values: ModbusValues,
    value: Arc<Value>,
}
struct GroupValue {
    name: String,
    values: ModbusValues,
}
enum ValueGroup {
    Sensor(Sensor),
    GroupPin(GroupPin),
    Group(GroupValue),
}

impl Sensor {
    pub fn new(s: SensorInit, values: ModbusValues, value: Option<Arc<Value>>) -> Self {
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
        SensorInit::GroupPin {name, pin, group_type:_typ}=> {
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
    
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn values(&self) -> &ModbusValues {
        &self.values
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

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
// use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ModbusSensors(HashMap<String, Arc<Sensor>>);

impl ModbusSensors {
    pub fn new() -> Self {
        ModbusSensors(HashMap::new())
    }
}


impl Deref for ModbusSensors {
    type Target = HashMap<String, Arc<Sensor>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ModbusSensors {
//     type Target = HashMap<String, Arc<Sensor>>;
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
