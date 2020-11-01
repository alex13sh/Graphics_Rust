use super::Builder;
// use std::hash::{Hash, Hasher};

// mod super::modbus_value;
// mod super::modbus_sensor;
use super::Value;
use super::{Sensor, SensorBuilder};

#[derive(Default, Debug)]
pub struct Device {
    name: String,
    values: Vec<Value>,
    sensors: Vec<Sensor>
}

enum Type {

}

// impl MudbusValue {
//     
// }

pub type DeviceBuilder = Builder<Device>;
impl Builder<Device> {
    pub fn name(mut self, value: String) -> Self {
        self.obj.name = value;
        self
    }
    
    fn push_value(mut self, value: &Value) -> Self {
//         self.obj.values.push(value);
        self
    }
    pub fn push_sensor(mut self, sensor: Sensor) -> Self {
//         for v in &sensor.values() {
//             self = self.push_value(v);
//         }
        
        self.obj.sensors.push(sensor);
        self
    }
    
    pub fn push_sensor_builder(mut self, sensor: SensorBuilder) -> Self {
//         for v in &sensor.values() {
//             self = self.push_value(v);
//         }
//         sensor.device(&self.obj);
        self.obj.sensors.push(sensor.complete());
        self
    }
}

