use super::{Device, DeviceInner, DeviceError};

use std::sync::Arc;

pub struct DigitIO {
    device: Arc<Device>, 
}

impl DeviceInner for DigitIO {
    fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
}

impl DigitIO {
    pub fn new(device: Device) -> Self {
        DigitIO {
            device: Arc::new(device)
        }
    }
    
    pub fn config_output(&self, pin: u8) {
        let sens = self.device.get_sensor_by_pin(pin);
        if let Some(sens) = sens {
            let vm = sens.values();
            let mut context = self.device.context().unwrap().borrow_mut();
            let v_conf = vm.get("Режим работы выхода").unwrap().clone();
            v_conf.update_value(0 as u32);
            context.set_value(&v_conf).unwrap();
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
}

impl DigitIO {
    pub fn config_genetaror_hz(&self, pin: u8) {
        let _vm = self.device.values_map();
        let sens = self.device.get_sensor_by_pin(pin);
//         if sens.is_none() {return;}
        if let Some(sens) = sens {
            let vm = sens.values();
            let mut context = self.device.context().unwrap().borrow_mut();
            // Настроить пин на шим.
            // Ну или сделать это в init модуле
            // 272 (0x110) + pin -- (1 - шим)
            let v_conf = vm.get("Режим работы выхода").unwrap().clone();
            v_conf.update_value(1 as u32);
            context.set_value(&v_conf).unwrap();
            // 308 (0x134) + pin -- (1000 ... 60 000) -- Период ШИМ выхода
            let v_interval = vm.get("Период ШИМ выхода").unwrap().clone();
            v_interval.update_value(1_000); // 1 Гц 
            context.set_value(&v_interval).unwrap();
            // 340 (0x154) + pin -- (0...1000)
            let v_procent = vm.get("Коэффициент заполнения ШИМ выхода").unwrap().clone();
            v_procent.update_value(500); // 50.0
            context.set_value(&v_procent).unwrap();
        }
    }
    
    pub fn set_hz(&self, hz: f32) -> Result<(), DeviceError> {
        let vm = self.device.values_map();
        let v_hz = vm.get("Test PWM/Период ШИМ выхода").unwrap().clone();
        v_hz.update_value((1_000_f32/hz) as u32);
        self.device.context()?.borrow_mut().set_value(&v_hz)?;
        Ok(())
    }
}

impl From<Device> for DigitIO {
    fn from(d: Device) -> Self {
        DigitIO {
            device: Arc::new(d)
        }
    }
}
