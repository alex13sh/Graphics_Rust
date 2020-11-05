// use super::Value;
use super::Sensor;

#[derive(Default, Debug)]
pub struct Device {
    pub name: String,
    pub sensors: Vec<Sensor>,
    pub device_type: DeviceType
}

pub enum DeviceType {
    OwenAnalog,
    OwenDigitalIO,
    // Немецкий модуль
}

impl DeviceType {
    pub fn new_sensor(&self) -> Sensor { // TODO: Изменить тип сенсора
        match *self {
            Self::OwenAnalog => {
                Sensor::default()
            },
            Self::OwenDigitalIO {
                Sensor::default()
            }
        }
    }
}
