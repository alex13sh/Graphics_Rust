use super::{Device, DeviceError};

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
            v_bitmap.set_bit(num-1, enb);
            self.device.context()?.borrow_mut().set_value(&v_bitmap)?;
            Ok(())
        } else {
            Err(DeviceError::ValueOut)
        }
    }
    pub fn get_turn_clapan(&self, num: u8)  ->  Result<bool, DeviceError> {
        if let 1..=8 = num  {
            let vm = self.device.values_map();
            let v_bitmap = vm.get("Битовая маска состояния выходов").unwrap().clone();
            self.device.context()?.borrow_mut().get_value(&v_bitmap)?;
            Ok(v_bitmap.get_bit(num-1))
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
