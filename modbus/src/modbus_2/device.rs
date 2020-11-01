// use super::Value;
use super::Sensor;

#[derive(Default, Debug)]
pub struct Device {
    pub name: String,
//     values: Vec<Value>,
    pub sensors: Vec<Sensor>
}
