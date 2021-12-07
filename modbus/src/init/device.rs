#![allow(dead_code)]

use super::Value;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub values: Vec<Value>,
    pub device_type: DeviceType<Device>, 
    pub address: DeviceAddress,
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
    pub(super) fn with_full_name_values(mut self) -> Self {
        for v in &mut self.values {
            if let Some(log) = &mut v.log {
                log.full_name = format!("{}/{}", self.name, v.name);
            }
        }
        self
    }
}
