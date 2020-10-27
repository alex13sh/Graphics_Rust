#[path = "./builder.rs"]
mod builder;
use builder::Builder;
// use std::hash::{Hash, Hasher};

use crate::modbus_value::ModbusValue;

#[derive(Default, Debug)]
pub struct ModbusDevice {
    name: String,
    values: Vec<ModbusValue>
}

enum Type {

}

// impl MudbusValue {
//     
// }

pub type ModbusDeviceBuilder = Builder<ModbusDevice>;
impl Builder<ModbusDevice> {
    pub fn name(mut self, value: String) -> Self {
        self.obj.name = value;
        self
    }
    
    pub fn push_value(mut self, value: ModbusValue) -> Self {
        self.obj.values.push(value);
        self
    }
}

