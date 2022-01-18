#![allow(dead_code)]

use super::Value;

#[derive(Debug)]
pub struct Device {
    pub name: DeviceID,
    pub values: Vec<Value>,
    pub address: DeviceAddress,
    
    pub config: DeviceConfig,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
pub struct DeviceConfig {
    // Интервал обновлений в секундах
    pub interval_update_in_sec: f32,
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
            v.name.device_id = Some(id);
            v.name.device_name = Some(self.name.name.clone());
            
            v.log = if let Some(log) = v.log.take() {
                Some(log.device(id, &self.name.name))
            } else {v.log.take()};
        }
        self
    }
}
