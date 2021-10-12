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
    pub fn low(invertor: modbus::Invertor, values: ModbusValues) -> Self {
        let temper_oil = ["Температура масла на верхн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1" ];
        HalfMeln {
            invertor: Invertor::from_device(invertor),
            motor: Motor::from(values.clone()),
            vibro: values.get_value_arc("Виброскорость").unwrap(),
            part: HalfPartInner::Low{
                temper_oil_values: values.get_values_by_name_contains(&temper_oil)
            },
            values: values,
        }
    }
    pub fn top(invertor: modbus::Invertor, values: ModbusValues) -> Self {
        // values.get_values_by_name_contains(
        let temp_podshib = ["Температура верх подшипника дв. М2", "Температура нижн подшипника дв. М2"];
        HalfMeln {
            invertor: Invertor::from_device(invertor),
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
    values: ModbusValues,
    device: Option<modbus::Invertor>,
    
    hz: ValueArc,
    
    amper: ValueArc,
//     volt: ValueArc,
}

impl Invertor {
    fn from_values(values: ModbusValues) -> Self {
    
        Invertor {
            hz: values.get_value_arc("Скорость двигателя").unwrap(),
            amper: values.get_value_arc("Выходной ток (A)").unwrap(),
            values: values,
            device: None,
        }
    }
    pub fn from_device(device: modbus::Invertor) -> Self {
        let values = device.device().values_map().clone();
        Self {
            hz: device.get_hz_out_value().into(),
            amper: device.get_amper_out_value().into(),
            device: Some(device),
            .. Self::from_values(values)
        }
    }
}

impl From<ModbusValues> for Invertor {
    fn from(values: ModbusValues) -> Invertor {
        Invertor::from_values(values)
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
pub enum SpeedChange {
    Acel, // Ускорение
    Plato, // Ускорение завершилось и скорость вышла на плато
    Decel, // Замедление
    Stop, // Остановка
}
