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
    
    pub fn turn_clapan(&self, name: &str, enb: bool)  ->  Result<(), DeviceError> {
        let vm = self.device.values_map();
//             let v_bitmap = vm.get("Битовая маска установки состояния выходов").unwrap().clone();
//             v_bitmap.set_bit(num-1, enb);
        if let Some(v) = vm.get(name) {
            v.set_bit(enb);
//             self.device.context()?.set_value(&v_bitmap)?;
            Ok(())
        } else {
            Err(DeviceError::ValueOut)
        }
    }
    pub fn get_turn_clapan(&self, name: &str)  ->  Result<bool, DeviceError> {
        let vm = self.device.values_map();
        if let Some(v) = vm.get(name) {
//             self.device.context()?.get_value(&v_bitmap)?;
            Ok(v.get_bit())
        } else {Err(DeviceError::ValueOut)}
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
