use super::Sensor;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub sensors: Option<Vec<Sensor>>,
//     pub values: Option<Vec<Value>>
    pub device_type: DeviceType
}

#[derive(Debug)]
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
        Self::OwenDigitalIO => {
            Sensor::default()
        }
        }
    }
}
