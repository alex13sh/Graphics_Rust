#![allow(dead_code)]

use super::init::{DeviceType, InvertorFunc};
use super::{Device, DeviceError};
use super::Value;

use std::sync::Arc;

pub struct DigitIO {
    device: Arc<Device>, 
}

impl DigitIO {
    pub fn new(device: Device) -> Self {
        DigitIO {
            device: Arc::new(device)
        }
    }
    
    pub fn turn_clapan(&self, num: u8, enb: bool)  ->  Result<(), DeviceError> {
        if let 1..=8 = num  {
            let vm = self.device.values_map();
            let v_bitmap = vm.get("Битовая маска установки состояния выходов").unwrap().clone();
            v_bitmap.set_bit(num, enb);
            self.device.context()?.borrow_mut().set_value(&v_bitmap);
            Ok(())
        } else {
            Err(DeviceError::ValueOut)
        }
    }
    
    
    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
}

impl From<Device> for DigitIO {
    fn from(d: Device) -> Self {
        DigitIO {
            device: Arc::new(d)
        }
    }
}
