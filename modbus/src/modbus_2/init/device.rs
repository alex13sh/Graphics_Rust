use super::ValueGroup;
use super::Value;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub sensors: Option<Vec<ValueGroup>>,
    pub values: Option<Vec<Value>>,
    pub device_type: DeviceType
}

#[derive(Debug)]
pub enum DeviceType {
    OwenAnalog,
    OwenDigitalIO,
    // Немецкий модуль
}
