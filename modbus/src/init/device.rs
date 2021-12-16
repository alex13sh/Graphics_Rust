#![allow(dead_code)]

use super::Value;

#[derive(Debug)]
pub struct Device {
    pub name: DeviceID,
    pub values: Vec<Value>,
    pub device_type: DeviceType<Device>, 
    pub address: DeviceAddress,
}

#[derive(Debug)]
pub struct DeviceID {
    pub id: u16, // Индекс начинается с 1
    pub name: String,
}

impl From<&str> for DeviceID {
    fn from(name: &str) -> Self {
        Self {
            id: 0,
            name: name.into(),
        }
    }
}

impl std::fmt::Display for DeviceID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}) {}", self.id, self.name)
    }
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

#[derive(Debug, Clone, Copy)]
pub enum InvertorFunc {
    DigitalInput(u8, u8), // Номер входа, Номер функции
    DigitalOutput(u8, u8),
    AnalogInput(u8, u8),
    AnalogOutput(u8, u8),
}

#[derive(Debug, Clone)]
pub enum DeviceAddress {
    TcpIP(String),
    Rtu(u8), // String device_convertor_name
    TcpIp2Rtu(String, u8), // String device_convertor_name
}

impl DeviceAddress {
    pub fn is_tcp_ip(&self) -> bool {
        if let DeviceAddress::TcpIP(_) = self {
            true
        } else {false}
    }
}

impl Device {
    pub fn with_id(mut self, id: u16) -> Self {
        self.name.id = id;
        for v in &mut self.values {
            v.name.device_id = id;
            v.name.device_name = self.name.name.clone();
            
            v.log = if let Some(log) = v.log.take() {
                Some(log.device(id, &self.name.name))
            } else {v.log.take()};
        }
        self
    }
}
