pub mod init {
use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DeviceResult, DigitIO};

use std::collections::BTreeMap;
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
//     pub digit_o: DigitIO,
    pub owen_analog_1: Arc<Device>,
    pub owen_analog_2: Arc<Device>,
    
    update_values: core::QueueUpdate,
}

impl Complect {
    pub fn new() -> Self {
        let invertor = init::make_invertor("192.168.1.5".into());
        let invertor = Invertor::new(invertor.into());
        let digit_i = DigitIO::new(init::make_io_digit("192.168.1.10".into()).into());
//         let digit_o = DigitIO::new(init::make_io_digit("192.168.1.12".into()).into());
        
        Complect {
            
            invertor: invertor,
            digit_i: digit_i,
//             digit_o: digit_o,
            
            owen_analog_1: Arc::new(Device::from(init::make_owen_analog_1("192.168.1.11"))),
            owen_analog_2: Arc::new(Device::from(init::make_owen_analog_2("192.168.1.13"))),
            
            update_values: core::QueueUpdate::empty(),
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
        &self.digit_i.device(), &self.invertor.device()]
        .iter().map(|&d| d.clone()).collect()
    }
    
    fn init_update_values(&mut self) {
        let map = self.make_values(false);
        for (k,v) in map {
            if v.is_bit() {
                println!("type is bit");
                self.update_values.append::<bool>(&k, &*v);
            } else {
                println!("type is float - {}", k);
                self.update_values.append::<f32>(&k, &*v);
            }
        }
    }
}
#[test]
fn test_init_update_values() {
    let mut c = Complect::new();
    c.init_update_values();
    assert!(false);
}

}
