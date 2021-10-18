#![allow(dead_code)]
use modbus::{ValueArc, ModbusValues, /*ValueError*/};

pub struct HalfMeln {
    values: ModbusValues, // Все значения

    pub invertor: Invertor,
    pub motor: Motor,
    pub vibro: ValueArc,
    part: HalfPartInner,
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
        let values = values.get_values_by_name_ends(&["М1"]) 
            + values.get_values_by_name_starts(&["5) Invertor"]);
        let температура_масла = ["Температура масла на верхн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1" ];
        HalfMeln {
            invertor: Invertor::from(&values),
            motor: Motor::from(&values),
            vibro: values.get_value_arc("Виброскорость").unwrap(),
            part: HalfPartInner::Low{
                температура_масла_values: values.get_values_by_name_contains(&температура_масла)
            },
            values: values,
        }
    }
    pub fn top(values: &ModbusValues) -> Self {
        let values = values.get_values_by_name_ends(&["М2"]) 
            + values.get_values_by_name_starts(&["6) Invertor"]);
        // values.get_values_by_name_contains(
        let температура_подшибника = ["Температура верх подшипника дв. М2", "Температура нижн подшипника дв. М2"];
        HalfMeln {
            invertor: Invertor::from(&values),
            motor: Motor::from(&values),
            vibro: values.get_value_arc("Виброскорость").unwrap(),
            part: HalfPartInner::Top{
                температура_подшибника_values: values.get_values_by_name_contains(&температура_подшибника)
            },
            values: values,
        }
    }
    pub fn get_part(&self) -> HalfPart {
        HalfPart::from(&self.part)
    }
}

pub type Invertor = modbus::InvertorValues;

pub struct Motor {
    pub speed: ValueArc,
    // speed_changed: Signal<SpeedChange>,
    
    // "Температура статора",
    // "Температура ротора Пирометр",
    температуры_values: ModbusValues, // Значения температур
}

impl From<&ModbusValues> for Motor {
    fn from(values: &ModbusValues) -> Self {
        Motor {
            speed: values.get_value_arc("Скорость двигателя").unwrap(),
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

pub mod watcher {
    use crate::Property;
    pub struct HalfMeln {
        invertor: Invertor,
        motor: Motor,
        
        vibro: Property<f32>,
        oil_temp: Property<f32>,
        
        speed_changed: Property<super::SpeedChange>, // Stop
        is_started: Property<bool>, // false
    }
    
    impl HalfMeln {
        pub(crate) fn update_property(&self, values: &super::HalfMeln) {
            use modbus::{Value, TryFrom};
            self.invertor.update_property(&values.invertor);
            let vibro: f32 = f32::try_from(&values.vibro as &Value).unwrap(); // todo: Необходимо обработать ошибку
            self.vibro.set(vibro);
        }
        
        async fn automation(&self) {
            let mut vibro = self.vibro.subscribe();
            let mut hz = self.invertor.hz.subscribe();
            let mut amper = self.invertor.amper.subscribe();
            loop {
                crate::changed_any!(vibro, hz, amper);
                {
                    let vibro = *vibro.borrow();
                    let hz = *hz.borrow();
                    let amper = *amper.borrow();
                    if self.is_started.get() == false 
                            && (hz > 1 || amper > 1) {
                        self.speed_changed.set(super::SpeedChange::Acel);
                        self.is_started.set(true);
                    } else if self.is_started.get() == true 
                            && hz < 2 && vibro < 0.2 && amper < 2 {
                        self.speed_changed.set(super::SpeedChange::Stop);
                        self.is_started.set(false);
                    }
                }
            }
        }
    }
    
    struct Invertor {
        hz: Property<u32>,
        speed: Property<u32>,
        amper: Property<u32>,
        volt: Property<u32>,
        watt: Property<u32>,
    }
    
    impl Invertor {
        fn update_property(&self, values: &super::Invertor) {
            let hz: u32 = values.get_hz_out_value().value();
            self.hz.set(hz);
            let amper: u32 = values.get_amper_out_value().value();
            self.amper.set(amper);
        }
    }
    
    struct Motor {
        speed: Property<u32>,
        
    }
}
