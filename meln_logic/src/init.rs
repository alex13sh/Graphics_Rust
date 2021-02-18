use super::{InvertorEngine, Properties};

use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DigitIO};

use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Complect {
    pub invertor_engine: InvertorEngine,
    
    pub invertor: Invertor,
    pub digit_io: DigitIO,
    pub owen_analog: Arc<Device>,
}

impl Complect {
    pub fn new() -> Self {
        let invertor = init::make_invertor("192.168.1.5".into());
        let invertor = Invertor::new(invertor.into());
        let digit_io = DigitIO::new(init::make_io_digit("192.168.1.10".into()).into());
        
        let dev_owen_analog: Device = init::make_owen_analog("192.168.1.11".into()).into();
        
        
        Complect {
            invertor_engine: InvertorEngine::new(),
            
            invertor: invertor,
            digit_io: digit_io,
            owen_analog: Arc::new(dev_owen_analog),
        }
    }
    pub fn make_values(&self) -> BTreeMap<String, Arc<Value>> {
        let dev_invertor = self.invertor.device();
        let dev_digit_io = self.digit_io.device();
        let dev_owen_analog = self.owen_analog.clone();
        
        let mut values = BTreeMap::new();
        for (dev, (k,v)) in dev_invertor.values_map().iter().map(|v|("Invertor", v))
            .chain(dev_digit_io.values_map().iter().map(|v|("DigitIO", v)))
            .chain(dev_owen_analog.values_map().iter().map(|v|("Analog", v)))
            .filter(|(_dev, (_k,v))| v.is_read_only()) {
            values.insert(format!("{}/{}", dev, k.clone()), v.clone());
        }
        values
    }
    
    pub fn update(&self) {
        use std::convert::TryFrom;
        let devices = [&self.owen_analog, 
            &self.digit_io.device(), &self.invertor.device()];
            
        for d in &devices {
            d.update();
        }
    }
    
    pub fn init_values(&self, values: &BTreeMap<String, Arc<Value>>) {
        println!("Values: {:?}", values.keys());
    }
}

#[test]
fn test_values() {
    let logic = Complect::new();
    let values = logic.make_values();
    logic.init_values(&values);
    assert!(false);
}
