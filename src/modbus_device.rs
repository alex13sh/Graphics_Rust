#[path = "./builder.rs"]
mod builder;
use builder::Builder;
// use std::hash::{Hash, Hasher};

// mod super::modbus_value;
// mod super::modbus_sensor;
use super::modbus_value::ModbusValue;
use super::modbus_sensor::ModbusSensor;

#[derive(Default, Debug)]
pub struct ModbusDevice <'a> {
    name: String,
    values: Vec<&'a ModbusValue>,
    sensors: Vec<ModbusSensor<'a>>
}

enum Type {

}

// impl MudbusValue {
//     
// }

pub type ModbusDeviceBuilder<'a> = Builder<ModbusDevice<'a>>;
impl <'a> Builder<ModbusDevice<'a>> {
    pub fn name(mut self, value: String) -> Self {
        self.obj.name = value;
        self
    }
    
    pub fn push_value(mut self, value: &'a ModbusValue) -> Self {
        self.obj.values.push(value);
        self
    }
    pub fn push_sensor(mut self, sensor: ModbusSensor<'a>) -> Self {
        for v in sensor.values().iter() {
            self = self.push_value(v);
        }
        
        self.obj.sensors.push(sensor);
        self
    }
}

