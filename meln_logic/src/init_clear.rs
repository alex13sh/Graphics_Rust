pub mod init {
use modbus::{Value, ModbusValues, /*ValueError*/};
use modbus::init;
use modbus::invertor::{Invertor, /*DvijDirect*/}; // Device
use modbus::{Device as MDevice, DeviceResult, DeviceError, UpdateReq, DigitIO};

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

macro_rules! map(
  { $T:ident, $($key:expr => $value:expr),+ } => {
    {
      let mut m = $T::new();
      $(
        m.insert($key, $value);
      )+
      m
    }
 };
);

use crate::devices::{Dozator};
type Device = Arc<MDevice>;
pub struct Complect {
    devices: Vec<Device>,
    values: ModbusValues,
}

// Инициализация
impl Complect {
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
        
        Complect {
            values: values,
            devices: devices,
        }
    }
    pub fn make_values(&self, read_only: bool) -> BTreeMap<String, Arc<Value>> {
        let devices = self.get_devices();//.map(|&d| d.clone());
        
        let mut values = BTreeMap::new();
        for (dev, (k,v)) in devices.iter()
            .flat_map(|d| {
                let dname = d.name().clone();
                d.values_map().iter()
                .map(move |(k,v)| (dname.clone(), (k,v)))
            }).filter(|(_d, (_k,v))| !read_only || v.is_read_only()) {
        
            values.insert(format!("{}/{}", dev, k.clone()), v.clone());
        }
        values
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
impl Complect {
    
    pub fn update(&self) {
        for d in self.get_devices() {
            d.update();
        }
    }
    pub fn update_all(&self) {
        for d in self.get_devices() {
            if let Err(err) = d.update_all() {
                println!("Device {} update error: {:?}", d.name(), err);
            }
        }
    }
    
    pub fn update_async(&self, req: UpdateReq) -> Vec<(/*device_id,*/ impl std::future::Future<Output = DeviceResult>)> {
        let mut device_futures = Vec::new();
        for d in self.get_devices() {
            if !d.is_connecting() && d.is_connect() {
                let d = d.clone();
                let upd = async move {
                    d.update_async(req).await
                };
                device_futures.push((upd));
            }
        }
        device_futures
    }
    pub fn reconnect_devices(&self) -> Vec<( impl std::future::Future<Output = DeviceResult>)> {
        let mut device_futures = Vec::new();
        for d in self.get_devices() {
            if !d.is_connecting() && !d.is_connect() {
                let d = d.clone();
                let upd = async move {d.connect().await};
                device_futures.push((upd));
            }
        }
        device_futures
    }
//     pub fn update_async_vibro(&self) -> Vec<(Arc<modbus::Device>,
//     impl std::future::Future<Output = DeviceResult>)> {
//         let d = self.owen_analog_2.clone();
//         let mut device_futures = Vec::new();
//         if !d.is_connecting() {
//             let  dc = d.clone();
//             let upd = async move {
//                 if !dc.is_connect() {
//                     let res = dc.connect().await;
//                     if res.is_err() {
//                         return res;
//                     }
//                 }
//                 dc.update_async(UpdateReq::Vibro).await
//             };
//             let dc = d.clone();
//             device_futures.push((dc, upd));
//         }
//         device_futures
//     }
    
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
    pub fn update_new_values_static(devices: &Vec<Arc<Device>>) -> DeviceResult {
        let mut res = Ok(());
        for d in devices {
            if let Err(e) = d.update_new_values() {
                res = Err(e);
            }
        }
        res
    }
}

// Изменения отделбных значений
impl Complect {
//     pub fn set_value(&self, name: &str, value: u32) {
    
    pub fn set_value(&self, name: &str, value: u32) -> DeviceResult {
        if let Some(v) = self.values.get(name) {
            v.set_value(value);
            Ok(())
        } else {Err(DeviceError::ValueOut)}
    }
    pub fn get_value(&self, name: &str) -> u32 {
        if let Some(v) = self.values.get(name) {
            v.value()
        } else {0}
    }
    pub fn get_valuef(&self, name: &str) -> f32 {
        use std::convert::TryFrom;
        if let Some(v) = self.values.get(name) {
            TryFrom::try_from(v.as_ref()).unwrap()
        } else {0.0}
    }
    pub fn set_bit(&self, name: &str, bit: bool) -> DeviceResult {
        if let Err(_) = self.values.set_bit(name, bit) {
            self.values.set_bit(&format!("{}/bit",name), bit)
                .map_err(|_| DeviceError::ValueOut)?;
        }
        Ok(())
    }
    pub fn get_bit(&self, name: &str) -> Result<bool, DeviceError> {
        return if let Ok(v) = self.values.get_bit(name) {
            Ok(v)
        } else {
            self.values.get_bit(&format!("{}/bit",name))
                .map_err(|_| DeviceError::ValueOut)
        }
    }
}
}
