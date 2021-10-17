use modbus::{ValueArc, ModbusValues};

pub struct Dozator {
    speed: ValueArc, // скоость ШИМа
    direct: ValueArc, 
}

impl Dozator {
    pub fn set_speed(&self, speed: i32) {
        self.direct.set_bit(speed >= 0);
        self.speed.set_value(speed.abs() as u32);
    }
}

impl From<&ModbusValues> for Dozator {
    fn from(values: &ModbusValues) -> Self {
        Dozator {
            speed: values.get_value_arc("Двигатель подачи материала в камеру/Частота высокочастотного ШИМ").unwrap(),
            direct: values.get_value_arc("Направление вращения двигателя ШД").unwrap(),
        }
    }
}

pub mod watcher {
    use crate::Property;
    pub struct Dozator {
        speed: Property<i32>,
    }
    impl Dozator {
        pub(crate) fn update_property(&self, values: &super::Dozator) {
            let mut speed: i32 = values.speed.value() as i32;
            if values.direct.get_bit() == false {
                speed = -speed;
            }
            self.speed.set(speed);
        }
    }
}
