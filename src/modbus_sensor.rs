#[path = "./builder.rs"]
mod builder;
use builder::Builder;
// use std::hash::{Hash, Hasher};

use super::modbus_value::ModbusValue;
use super::modbus_device::ModbusDevice;

#[derive(Default, Debug)]
pub struct ModbusSensor<'a> {
    name: String,
    values: Vec<ModbusValue>,
    device: Option<&'a ModbusDevice<'a>>
}

enum Type {

}

impl<'a> ModbusSensor<'a> {

    pub fn values(&'a self) -> &'a Vec<ModbusValue> {
        &self.values
    }
    
}

pub type ModbusSensorBuilder<'a> = Builder<ModbusSensor<'a>>;
impl <'a> Builder<ModbusSensor<'a>> {
    pub fn name(mut self, value: String) -> Self {
        self.obj.name = value;
        self
    }
    
    pub fn push_value(mut self, value: ModbusValue) -> Self {
        self.obj.values.push(value);
        self
    }
}

