use super::ValueGroup;
use super::Value;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub sensors: Option<Vec<ValueGroup>>,
    pub values: Option<Vec<Value>>,
    pub device_type: DeviceType<Device>, 
//     pub address: DeviceAddress,
}

#[derive(Debug)]
pub enum DeviceType<D> {
    OwenAnalog,
    OwenDigitalIO,
    // Немецкий модуль
    Invertor {
        functions: Vec<InvertorFunc>
    },
    Convertor {
        devices: Vec<D>
    }
}

#[derive(Debug)]
pub enum InvertorFunc {
    DigitalInput(u8, u8), // Номер входа, Номер функции
    DigitalOutput(u8, u8),
    AnalogInput(u8, u8),
    AnalogOutput(u8, u8),
}

#[derive(Debug)]
pub enum DeviceAddress {
    TcpIp(String),
    Rtu(u8), // String device_convertor_name
}
