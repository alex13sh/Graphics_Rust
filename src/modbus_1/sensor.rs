use super::Builder;
// use std::hash::{Hash, Hasher};
// use core::iter::*;

use super::Value;
use super::Device;

#[derive(Default, Debug)]
pub struct Sensor {
    name: String,
    values: Vec<Value>,
    device: Option<Device>
}

enum Type {

}

impl Sensor {

//     pub fn values(&self) -> Vec<ModbusValue> {
//         self.values
//     }
    
}

pub type SensorBuilder = Builder<Sensor>;
impl Builder<Sensor> {
    pub fn name(mut self, value: String) -> Self {
        self.obj.name = value;
        self
    }
    
    pub fn push_value(mut self, value: Value) -> Self {
        self.obj.values.push(value);
        self
    }
    
//     pub fn device(mut self, device: &Device) -> Self {
//         self.obj.device = Some(device);
//         self
//     }
}
