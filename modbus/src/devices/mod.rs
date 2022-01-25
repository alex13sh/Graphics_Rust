pub mod invertor;
pub use invertor::*;

use super::init;
use super::{Device as MDevice, DeviceResult, DeviceError, UpdateReq};
use super::{ModbusValues, Value};

use std::collections::HashMap;
use std::sync::Arc;

type Device = Arc<MDevice>;
pub struct Devices {
    devices: Vec<Device>,
    values: ModbusValues,
}

// Инициализация
impl Devices {
    pub fn new() -> Self {
        let devices: Vec<_> = vec![
            init::make_invertor("192.168.1.5".into()).with_id(5),
            init::make_invertor("192.168.1.6".into()).with_id(6),
        
            init::make_i_digit("192.168.1.10".into()).with_id(3), 
            init::make_o_digit("192.168.1.12".into()).with_id(4),
            init::make_owen_analog_1("192.168.1.11").with_id(1),
            init::make_owen_analog_2("192.168.1.13", 11).with_id(2),
            init::make_pdu_rs("192.168.1.13", 12).with_id(7),
            init::make_mkon("192.168.1.13", 1),
        ].into_iter().map(MDevice::from).map(Arc::new).collect();
        
        let values = Self::init_values(&devices);
        
        Devices {
            values: values,
            devices: devices,
        }
    }

    pub fn get_values(&self) -> &ModbusValues {
        &self.values
    }
    
    fn init_values(devices: &[Device]) -> ModbusValues {
        let mut map = HashMap::new();
        for d in devices.iter() {
            for (id, v) in d.values_map().iter() {
                map.insert(id.clone(), v.clone());
            }
        }
//         dbg!(map.keys());
        ModbusValues::from(map)
    }
}

// Обновление всех устройств
impl Devices {
    pub fn iter(&self) -> impl Iterator<Item=&Device> {
        self.devices.iter()
    }
}
