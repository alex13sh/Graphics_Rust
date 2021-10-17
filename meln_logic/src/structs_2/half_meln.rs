use modbus::{Value, ValueArc, ModbusValues, ValueError};

pub struct HalfMeln {
    values: ModbusValues, // Все значения

    pub invertor: Invertor,
    pub motor: Motor,
    pub vibro: ValueArc,
    part: HalfPartInner,
}

enum HalfPartInner {
    Low {
        temper_oil_values: ModbusValues,
    }, 
    Top {
        temp_podshib_values: ModbusValues,
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
    pub fn low(values: ModbusValues) -> Self {
        let temper_oil = ["Температура масла на верхн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1" ];
        HalfMeln {
            invertor: Invertor::from_values(&values),
            motor: Motor::from(values.clone()),
            vibro: values.get_value_arc("Виброскорость").unwrap(),
            part: HalfPartInner::Low{
                temper_oil_values: values.get_values_by_name_contains(&temper_oil)
            },
            values: values,
        }
    }
    pub fn top(values: ModbusValues) -> Self {
        // values.get_values_by_name_contains(
        let temp_podshib = ["Температура верх подшипника дв. М2", "Температура нижн подшипника дв. М2"];
        HalfMeln {
            invertor: Invertor::from_values(&values),
            motor: Motor::from(values.clone()),
            vibro: values.get_value_arc("Виброскорость").unwrap(),
            part: HalfPartInner::Top{
                temp_podshib_values: values.get_values_by_name_contains(&temp_podshib)
            },
            values: values,
        }
    }
    pub fn get_part(&self) -> HalfPart {
        HalfPart::from(&self.part)
    }
}

pub struct Invertor {
    values: modbus::InvertorValues,
}

impl Invertor {
    fn from_values(values: &ModbusValues) -> Self {
    
        Invertor {
            values: modbus::InvertorValues::from_values(values),
        }
    }
//     pub fn from_device(device: modbus::Invertor) -> Self {
//         Invertor {
//             values: device::InvertorValues::from_device(&device),
//         }
//     }
}

impl From<ModbusValues> for Invertor {
    fn from(values: ModbusValues) -> Invertor {
        Invertor::from_values(&values)
    }
}

pub struct Motor {
    pub speed: ValueArc,
    // speed_changed: Signal<SpeedChange>,
    
    // "Температура статора",
    // "Температура ротора Пирометр",
    temper_values: ModbusValues, // Значения температур
}

impl Motor {
    fn from_values(values: ModbusValues) -> Self {
        Motor {
            speed: values.get_value_arc("Скорость двигателя").unwrap(),
            temper_values: values.get_values_by_name_contains(&["Температура статора", "Температура ротора Пирометр",]),
        }
    }
}

impl From<ModbusValues> for Motor {
    fn from(values: ModbusValues) -> Motor {
        Motor::from_values(values)
    }
}

// Сообщение об изменение скорости
#[derive(Debug)]
pub enum SpeedChange {
    Acel, // Ускорение
    Plato, // Ускорение завершилось и скорость вышла на плато
    Decel, // Замедление
    Stop, // Остановка
}

mod watcher {
    use crate::Property;
    struct HalfMeln {
        invertor: Invertor,
        motor: Motor,
        
        vibro: Property<f32>,
        oil_temp: Property<f32>,
        
        speed_changed: Property<super::SpeedChange>, // Stop
        is_started: Property<bool>, // false
    }
    
    impl HalfMeln {
        fn update_property(&self, values: &super::HalfMeln) {
            self.invertor.update_property(&values.invertor);
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
                        self.speed_changed.send(super::SpeedChange::Acel);
                        self.is_started.set(true);
                    } else if self.is_started.get() == true 
                            && hz < 2 && vibro < 0.2 && amper < 2 {
                        self.speed_changed.send(super::SpeedChange::Stop);
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
            let hz: u32 = values.values.get_hz_out_value().value();
            self.hz.set(hz);
            let amper: u32 = values.values.get_amper_out_value().value();
            self.amper.set(amper);
        }
    }
    
    struct Motor {
        speed: Property<u32>,
        
    }
}
