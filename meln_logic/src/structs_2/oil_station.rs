use modbus::{Value, ValueArc, ModbusValues};

pub struct OilStation {
    temp: ValueArc,
    motor: ValueArc,
}

impl From<&ModbusValues> for OilStation {
    fn from(values: &ModbusValues) -> Self {
        OilStation {
            temp: values.get_value_arc("Температура масла на выходе маслостанции").unwrap(),
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
    pub fn temper(&self) -> f32 {
        use modbus::TryFrom;
        f32::try_from(&self.temp as &Value).unwrap() // todo: Обработка ошибок
    }
}

pub mod watcher {
    use crate::Property;
    pub struct OilStation {
        pub temp: Property<f32>,
        pub motor: Property<bool>,
    }
    
    impl OilStation {
        pub(crate) fn update_property(&self, values: &super::OilStation) {
            self.temp.set(values.temper());
            self.motor.set(values.motor.get_bit());
        }
    }
}
