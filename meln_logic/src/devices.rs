use modbus::{init, ModbusValues, /*ValueError*/};
use modbus::{Device as MDevice, DeviceResult, DeviceError, UpdateReq};

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
            for (name, v) in d.values_map().iter() {
                if let Some(_) = map.insert(name.clone(), v.clone()) {
                    map.remove(name.as_str());
                }
                map.insert(format!("{}/{}", d.name(), name), v.clone());
            }
        }
//         dbg!(map.keys());
        ModbusValues::from(map)
    }
}

// Обновление всех устройств
impl Devices {
        
    /*device_id, -- вместо Arc<Device>*/ 
    pub fn update_async(&self, req: UpdateReq) -> Vec<(Device, impl std::future::Future<Output = DeviceResult>)> {
        let mut device_futures = Vec::new();
        for d in self.get_devices() {
            if !d.is_connecting() && d.is_connect() {
                let upd = d.clone().update_async(req);
                device_futures.push((d.clone(), upd));
            }
        }
        device_futures
    }
    pub fn reconnect_devices(&self) -> Vec<(Device, impl std::future::Future<Output = DeviceResult>)> {
        let mut device_futures = Vec::new();
        for d in self.get_devices() {
            if !d.is_connecting() && !d.is_connect() {
                let upd = d.clone().connect();
                device_futures.push((d.clone(), upd));
            }
        }
        device_futures
    }
    
    pub fn get_devices(&self) -> &[Device] {
        &self.devices
    }
    
    pub fn update_new_values(&self) -> DeviceResult {
        let mut res = Ok(());
        for d in self.get_devices() {
            if let Err(e) = d.update_new_values() {
                res = Err(e);
            }
        }
        res
    }

}
