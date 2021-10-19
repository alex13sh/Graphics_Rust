#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};

pub struct VacuumStation {
    vacuum: ValueArc,
    
    motor_1: ValueArc,
    motor_2: ValueArc,
    
    клапан_напуска: ValueArc,
    клапан_насоса: ValueArc,
}

impl From<&ModbusValues> for VacuumStation {
    fn from(values: &ModbusValues) -> Self {
        VacuumStation {
            vacuum: values.get_value_arc("Разрежение воздуха в системе").unwrap(),
            motor_1: values.get_value_arc("Двигатель насоса вакуума 1").unwrap(),
            motor_2: values.get_value_arc("Двигатель насоса вакуума 2").unwrap(),
            клапан_напуска: values.get_value_arc("Клапан напуска").unwrap(),
            клапан_насоса: values.get_value_arc("Клапан насоса М5").unwrap(),
        }
    }
}

impl VacuumStation {
    // Уменьшить давление
    pub fn davl_down(&self) {
        self.motor_1.set_bit(true);
        self.motor_2.set_bit(true);
        // Добавить задержку
        self.клапан_насоса.set_bit(true);
    }
    // Отключить насосы
    pub fn davl_dis(&self) {
        self.клапан_насоса.set_bit(false);
        // Добавить задержку
        self.motor_1.set_bit(false);
        self.motor_2.set_bit(false);
        
        self.клапан_напуска.set_bit(false);
    }
    // Увеличить давление
    pub fn davl_up(&self) {
        self.davl_dis();
        
        self.клапан_напуска.set_bit(true);
    }
}

pub mod watcher {
    use crate::structs::Property;
    pub struct VacuumStation {
        pub vacuum: Property<f32>,
        
        pub motor: Property<bool>,
        
        pub клапан_напуска: Property<bool>,
        pub клапан_насоса: Property<bool>,
    }
    
    impl VacuumStation {
        pub(crate) fn update_property(&self, values: &super::VacuumStation) {
            use modbus::{Value, TryFrom};
            let vacuum: f32 = f32::try_from(&values.vacuum as &Value).unwrap(); // todo: Обработка ошибок
            self.vacuum.set(vacuum);
            self.motor.set(
                values.motor_1.get_bit()
                && values.motor_2.get_bit()
            );
            self.клапан_напуска.set(values.клапан_напуска.get_bit());
            self.клапан_насоса.set(values.клапан_насоса.get_bit());
        }
    }
}
