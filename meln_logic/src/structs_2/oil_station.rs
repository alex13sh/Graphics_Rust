#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};

pub struct OilStation {
    температура: ValueArc,
    motor: ValueArc,
}

impl From<&ModbusValues> for OilStation {
    fn from(values: &ModbusValues) -> Self {
        OilStation {
            температура: values.get_value_arc("Температура масла на выходе маслостанции").unwrap(),
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
    pub fn температура(&self) -> f32 {
        use modbus::{Value, TryFrom};
        f32::try_from(&self.температура as &Value).unwrap() // todo: Обработка ошибок
    }
}

pub mod watcher {
    use crate::Property;
    pub struct OilStation {
        pub температура: Property<f32>,
        pub motor: Property<bool>,
    }
    
    impl OilStation {
        pub(crate) fn update_property(&self, values: &super::OilStation) {
            self.температура.set(values.температура());
            self.motor.set(values.motor.get_bit());
        }
    }
}
