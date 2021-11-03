#![allow(dead_code)]
use modbus::{ValueArc, ModbusValues, /*ValueError*/};

pub struct HalfMeln {
    values: ModbusValues, // Все значения

    pub invertor: Invertor,
    pub motor: Motor,
    pub vibro: ValueArc,
    part: HalfPartInner,
    pub температура_верх: ValueArc,
    pub температура_нижн: ValueArc,
}

enum HalfPartInner {
    Low {
        температура_масла_values: ModbusValues,
    }, 
    Top {
        температура_подшибника_values: ModbusValues,
    }, 
}
pub enum HalfPart {
    Low,
    Top,
}

impl From<&HalfPartInner> for HalfPart {
    fn from(part: &HalfPartInner) -> Self {
        match part {
        HalfPartInner::Low{..} => HalfPart::Low,
        HalfPartInner::Top{..} => HalfPart::Top,
        }
    }
}

impl HalfMeln {
    pub fn low(values: &ModbusValues) -> Self {
        let values = values.get_values_by_name_contains(&["М1/"]).convert_to_sensor()
            + values.get_values_by_name_start("5) Invertor/");
        let температура_масла = ["Температура масла на верхн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1" ];
        HalfMeln {
            invertor: Invertor::from(&values),
            motor: Motor::from(&values),
            vibro: values.get_value_arc_starts("Виброскорость").unwrap(),
            part: HalfPartInner::Low{
                температура_масла_values: values.get_values_by_name_contains(&температура_масла)
            },
            температура_верх: values.get_value_arc(температура_масла[0]).unwrap(),
            температура_нижн: values.get_value_arc(температура_масла[1]).unwrap(),
            values: values,
        }
    }
    pub fn top(values: &ModbusValues) -> Self {
        let values = values.get_values_by_name_contains(&["М2/"]).convert_to_sensor()
            + values.get_values_by_name_start("6) Invertor/");
        let температура_подшибника = ["Температура верх подшипника дв. М2", "Температура нижн подшипника дв. М2"];
        HalfMeln {
            invertor: Invertor::from(&values),
            motor: Motor::from(&values),
            vibro: values.get_value_arc_starts("Виброскорость").unwrap(),
            part: HalfPartInner::Top{
                температура_подшибника_values: values.get_values_by_name_contains(&температура_подшибника)
            },
            температура_верх: values.get_value_arc(температура_подшибника[0]).unwrap(),
            температура_нижн: values.get_value_arc(температура_подшибника[1]).unwrap(),
            values: values,
        }
    }
    pub fn get_part(&self) -> HalfPart {
        HalfPart::from(&self.part)
    }
    
    pub fn stop(&self) {
        self.invertor.stop();
    }
}

pub type Invertor = modbus::InvertorValues;

pub struct Motor {
//     speed: ValueArc,
    // speed_changed: Signal<SpeedChange>,
    
    pub температура_статора: ValueArc, // "Температура статора",
    pub температура_ротора: ValueArc, // "Температура ротора Пирометр",
    
    температуры_values: ModbusValues, // Значения температур
}

impl From<&ModbusValues> for Motor {
    fn from(values: &ModbusValues) -> Self {
        Motor {
//             speed: values.get_value_arc("Скорость двигателя").unwrap(),
            температура_статора: values.get_value_arc_starts("Температура статора").unwrap(),
            температура_ротора: values.get_value_arc_starts("Температура ротора Пирометр").unwrap(),
            температуры_values: values.get_values_by_name_contains(&["Температура статора", "Температура ротора Пирометр",]),
        }
    }
}

// Сообщение об изменение скорости
#[derive(Debug, PartialEq)]
pub enum SpeedChange {
    Acel, // Ускорение
    Plato, // Ускорение завершилось и скорость вышла на плато
    Decel, // Замедление
    Stop, // Остановка
}

impl Default for SpeedChange {
    fn default() -> Self {
        SpeedChange::Stop
    }
}

pub mod watcher {
//     #[macro_use]
    use crate::structs::{Property, changed_any};
    
    #[derive(Default)]
    pub struct HalfMeln {
        pub invertor: Invertor,
        pub motor: Motor,
        
        pub vibro: Property<f32>,
        pub oil_temp: Property<f32>,
        
        pub speed_changed: Property<super::SpeedChange>, // Stop
        pub is_started: Property<bool>, // false
    }
    
    impl HalfMeln {
        pub(crate) fn update_property(&self, values: &super::HalfMeln) {
            use modbus::{Value, TryFrom};
            self.invertor.update_property(&values.invertor);
            if let Ok(vibro) = f32::try_from(&values.vibro as &Value) { // todo: Необходимо обработать ошибку
                self.vibro.set(vibro);
            }
        }
        
        pub(crate) async fn automation(&self) {
            let mut vibro = self.vibro.subscribe();
            let mut speed = self.invertor.speed.subscribe();
            let mut amper = self.invertor.amper.subscribe();
            loop {
                changed_any!(vibro, speed, amper);
                {
                    let vibro = *vibro.borrow();
                    let speed = *speed.borrow();
                    let amper = *amper.borrow();
                    let is_started = self.is_started.get();
                    if is_started  == false
                            && (speed > 1 || amper > 1) {
                        self.speed_changed.set(super::SpeedChange::Acel);
                        self.is_started.set(true);
                    } else if is_started  == true
                            && speed < 2 && vibro < 0.15 && amper < 2 {
                        self.speed_changed.set(super::SpeedChange::Stop);
                        self.is_started.set(false);
                    }
                }
            }
        }
    }
    
    #[derive(Default)]
    pub struct Invertor {
        pub hz: Property<u32>,
        pub speed: Property<u32>,
        pub amper: Property<u32>,
        pub volt: Property<u32>,
        pub watt: Property<u32>,
    }
    
    impl Invertor {
        fn update_property(&self, values: &super::Invertor) {
            let hz: u32 = values.get_hz_out_value().value();
            self.hz.set(hz);
            let speed: u32 = values.get_speed_out_value().value();
            self.speed.set(speed);
            let amper: u32 = values.get_amper_out_value().value();
            self.amper.set(amper);
        }
    }
    
    #[derive(Default)]
    pub struct Motor {
        pub speed: Property<u32>,
        
    }
}
