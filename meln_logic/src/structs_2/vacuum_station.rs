use modbus::{Value, ValueArc, ModbusValues};

pub struct VacuumStation {
    vacuum: ValueArc,
    
    motor_1: ValueArc,
    motor_2: ValueArc,
    
    klp_napusk: ValueArc,
    klp_nasos: ValueArc,
}

impl From<&ModbusValues> for VacuumStation {
    fn from(values: &ModbusValues) -> Self {
        VacuumStation {
            vacuum: values.get_value_arc("Разрежение воздуха в системе").unwrap(),
            motor_1: values.get_value_arc("Двигатель насоса вакуума 1").unwrap(),
            motor_2: values.get_value_arc("Двигатель насоса вакуума 2").unwrap(),
            klp_napusk: values.get_value_arc("Клапан напуска").unwrap(),
            klp_nasos: values.get_value_arc("Клапан насоса М5").unwrap(),
        }
    }
}

impl VacuumStation {
    pub fn davl_down(&self) {
        self.motor_1.set_bit(true);
        self.motor_2.set_bit(true);
        // Добавить задержку
        self.klp_nasos.set_bit(true);
    }
    pub fn davl_dis(&self) {
        self.klp_nasos.set_bit(false);
        // Добавить задержку
        self.motor_1.set_bit(false);
        self.motor_2.set_bit(false);
        
        self.klp_napusk.set_bit(false);
    }
    pub fn davl_up(&self) {
        self.davl_dis();
        
        self.klp_napusk.set_bit(true);
    }
}

pub mod watcher {
    use crate::Property;
    pub struct VacuumStation {
        pub vacuum: Property<f32>,
        
        pub motor: Property<bool>,
        
        pub klp_napusk: Property<bool>,
        pub klp_nasos: Property<bool>,
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
            self.klp_napusk.set(values.klp_napusk.get_bit());
            self.klp_nasos.set(values.klp_nasos.get_bit());
        }
    }
}
