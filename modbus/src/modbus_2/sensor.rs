use super::Value;
// use super::Device;

#[derive(Default, Debug)]
pub struct Sensor {
    pub name: String,
    pub values: Vec<Value>,
//     device: Option<Device>
}
