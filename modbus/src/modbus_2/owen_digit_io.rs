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
        let vm = self.device.values_map();
        
        Ok(())
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
