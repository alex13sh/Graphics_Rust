#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};

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
            уровень_масла: values.get_value_arc("PDU-RS/value").unwrap(),
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
    pub fn температура(&self) -> f32 {
        use modbus::{Value, TryFrom};
        f32::try_from(&self.температура as &Value).unwrap() // todo: Обработка ошибок
    }
    pub fn давление_масла(&self) -> f32 {
        use modbus::{Value, TryFrom};
        f32::try_from(&self.давление_масла as &Value).unwrap() // todo: Обработка ошибок
    }
}

pub mod watcher {
    use crate::structs::Property;
    
    #[derive(Default)]
    pub struct OilStation {
        pub температура: Property<f32>,
        pub давление_масла: Property<f32>,
        pub уровень_масла: Property<u32>,
        pub motor: Property<bool>,
    }
    
    impl OilStation {
        pub(crate) fn update_property(&self, values: &super::OilStation) {
            self.температура.set(values.температура());
            self.давление_масла.set(values.давление_масла());
            self.уровень_масла.set(values.уровень_масла.value());
            self.motor.set(values.motor.get_bit());
        }
    }
}
