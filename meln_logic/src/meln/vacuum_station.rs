#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};

pub struct VacuumStation {
    pub vacuum: ValueArc,
    
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
        // Если включён клапан напуска
        self.davl_dis();

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
        // Если включёны насосы
        self.davl_dis();
        
        self.клапан_напуска.set_bit(true);
    }
}

pub mod watcher {
    use crate::watcher::{Property, changed_any};
    
    #[derive(Default)]
    pub struct VacuumStation {
        pub vacuum: Property<f32>,
        
        pub motor: Property<bool>,
        
        pub клапан_напуска: Property<bool>,
        pub клапан_насоса: Property<bool>,

        pub status: Property<Status>,
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

        pub(crate) async fn automation(&self) {
            loop {
                self.status.set(
                    Status::new(self).await
                        .unwrap_or(self.status.get())
                );
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    #[allow(non_camel_case_types)]
    pub enum Status {
        Насосы_отключены,
        Уменьшение_давления,
        Увеличение_давления,
    }

    impl Default for Status {
        fn default() -> Self {
            Status::Насосы_отключены
        }
    }

    impl Status {
        async fn new(vacuum: &VacuumStation) -> Option<Self> {
            let mut motor = vacuum.motor.subscribe();
            let mut клапан_насоса = vacuum.клапан_насоса.subscribe();
            let mut клапан_напуска = vacuum.клапан_напуска.subscribe();
            // Проверка отрабатывает несколько раз из-за того, что мотор и клапан одновременно меняется.
//             changed_any!(motor, клапан_насоса, клапан_напуска);
            // Более тонкая настройка ожиданий избавляет от лишних проверок.
            tokio::select! {
            _ = futures_util::future::join_all(vec![motor.changed(), клапан_насоса.changed()]) => {},
            _ = клапан_напуска.changed() => {},
            };
            let status = match (motor.borrow().clone(), клапан_насоса.borrow().clone(), клапан_напуска.borrow().clone()) {
            (false, false, false) => Some(Status::Насосы_отключены),
            (true, true, false) => Some(Status::Уменьшение_давления),
            (false, false, true) => Some(Status::Увеличение_давления),
            _ => None,
            };
            log::trace!("new status: {:?}", &status);
            status
        }
    }
}
