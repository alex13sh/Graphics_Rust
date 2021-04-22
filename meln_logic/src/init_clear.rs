pub mod init {
use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DeviceResult, DeviceError, DigitIO};

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

pub struct Complect {
        
    pub invertor: Invertor,
    pub digit_i: DigitIO,
    pub digit_o: DigitIO,
    pub owen_analog_1: Arc<Device>,
    pub owen_analog_2: Arc<Device>,
    
    values: ModbusValues,
}

impl Complect {
    pub fn new() -> Self {
        let invertor = init::make_invertor("192.168.1.5".into());
        let invertor = Invertor::new(invertor.into());
        let digit_i = DigitIO::new(init::make_i_digit("192.168.1.10".into()).into());
        let digit_o = DigitIO::new(init::make_o_digit("192.168.1.12".into()).into());
        let analog_1 = Arc::new(Device::from(init::make_owen_analog_1("192.168.1.11")));
        let analog_2 = Arc::new(Device::from(init::make_owen_analog_2("192.168.1.13")));
        Complect {
            values: Self::init_values(&mut [&invertor.device(), &digit_i.device(), &digit_o.device(), &analog_1, &analog_2]),
            
            invertor: invertor,
            digit_i: digit_i,
            digit_o: digit_o,
            owen_analog_1: analog_1,
            owen_analog_2: analog_2,
            
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
    
    pub fn update(&self) {
        for d in &self.get_devices() {
            d.update();
        }
    }
    pub fn update_all(&self) {
        for d in &self.get_devices() {
            if let Err(err) = d.update_all() {
                println!("Device {} update error: {:?}", d.name(), err);
            }
        }
    }
    
    pub fn update_async(&self) -> Vec<(Arc<modbus::Device>,
    impl std::future::Future<Output = DeviceResult>)> {
        let mut device_futures = Vec::new();
        for d in &self.get_devices() {
            if !d.is_connecting() {
                let  dc = d.clone();
                let upd = async move {
                    if !dc.is_connect() {
                        let res = dc.connect().await;
                        if res.is_err() {
                            return res;
                        }
                    }
                    dc.update_async().await
                };
                let dc = d.clone();
                device_futures.push((dc, upd));
            }
        }
        device_futures
    }
    
    pub fn get_devices(&self) -> Vec<Arc<Device>> {
        [&self.owen_analog_1, &self.owen_analog_2,
        &self.digit_i.device(), &self.digit_o.device(), 
        &self.invertor.device()]
        .iter().map(|&d| d.clone()).collect()
    }
    
    fn init_values(devices: &mut [&Device]) -> ModbusValues {
        let map: HashMap<_,_> = devices.iter().flat_map(|d|d.values_map().iter())
            .map(|(name, v)| (name.clone(), v.clone()))
            .collect();
        ModbusValues::from(map)
    }
    
    pub fn update_new_values(&self) -> DeviceResult {
        for d in self.get_devices() {
            d.update_new_values()?;
        }
        Ok(())
    }
    
    pub fn set_value(&self, name: &str, value: u32) {
        if let Some(v) = self.values.get(name) {
            v.set_value(value);
        }
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
        let v = self.values.get(name);
        let v = if v.is_some() {v} else {
            self.values.get(&format!("{}/bit",name))
        };
        
        if let Some(v) = v {
            v.set_bit(bit);
            Ok(())
        } else {Err(DeviceError::ValueOut)}
    }
    pub fn get_bit(&self, name: &str) -> Result<bool, DeviceError> {
        let v = self.values.get(name);
        let v = if v.is_some() {v} else {self.values.get(&format!("{}/bit",name))};
        
        if let Some(v) = v {
            Ok(v.get_bit())
        } else {Err(DeviceError::ValueOut)}
    }
}
}