#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};
use modbus::ValueFloatError;

pub struct OilStation {
    pub температура: ValueArc,
    pub давление_масла: ValueArc,
    pub уровень_масла: ValueArc,
    motor: ValueArc,
}

impl From<&ModbusValues> for OilStation {
    fn from(values: &ModbusValues) -> Self {
        OilStation {
            температура: values.get_value_arc("Температура масла на выходе маслостанции").unwrap(),
            давление_масла: values.get_value_arc("Давление масла на выходе маслостанции").unwrap(),
            уровень_масла: values.get_value_arc("Процентное значение уровня масла").unwrap(),
            motor: values.get_value_arc("Двигатель маслостанции М4").unwrap(),
        }
    }
}

impl OilStation {
    pub fn start(&self) {
        self.motor.set_bit(true);
    }
    pub fn stop(&self) {
        self.motor.set_bit(false);
    }
    pub fn motor_turn(&self, enb: bool) {
        self.motor.set_bit(enb);
    }
    pub fn температура(&self) -> Result<f32, ValueFloatError> {
        use modbus::{Value, TryFrom};
        f32::try_from(&self.температура as &Value)
    }
    pub fn давление_масла(&self) -> Result<f32, ValueFloatError> {
        use modbus::{Value, TryFrom};
        f32::try_from(&self.давление_масла as &Value)
    }
}

pub mod watcher {
    use crate::watcher::Property;

    use bitflags::bitflags;
    bitflags! {
        #[derive(Default)]
        pub struct OilStationError: u8 {
            const НедостаточноДавленияМасла = 1<<0;
            const ВысокаяТемператураМасла = 1<<1;
        }
    }

    #[derive(Default, Debug)]
    pub struct OilStation {
        pub температура: Property<f32>,
        pub давление_масла: Property<f32>,
        pub уровень_масла: Property<u32>,
        pub motor: Property<bool>,

        // TODO: Использовать в отображении статуса аппарата
        pub error: Property<OilStationError>,
    }
    
    impl OilStation {
        pub(crate) fn update_property(&self, values: &super::OilStation) {
            let mut err_flags = self.error.get();
            if let Ok(v) = values.температура() {
                self.температура.set(v);
            }
            err_flags.set(OilStationError::ВысокаяТемператураМасла, values.температура.is_error());
            if let Ok(v) = values.давление_масла() {
                self.давление_масла.set(v);
            }
            err_flags.set(OilStationError::НедостаточноДавленияМасла, values.давление_масла.is_error());
            self.error.set(err_flags);

            self.уровень_масла.set(values.уровень_масла.value());
//             log::trace!("motor get_bit: {}", values.motor.get_bit());
            self.motor.set(values.motor.get_bit());
        }
    }
}
