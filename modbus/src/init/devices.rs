use super::{Value, ValueSize, ValueDirect, ValueError};
use super::Log;

pub(super) fn make_value(prefix: &str, name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Value {
    super::make_value(&format!("{}/{}", prefix, name), address, size, direct)
}
pub mod owen_analog {
    use super::*;
    pub fn make_sensor(pin: u8, name: &str, err: ValueError, val_size: ValueSize) -> Vec<Value> {
        let pin = pin as u16 - 1;
        let prefix = name;
        vec![
            Value {
                log: Log::hash(&format!("{}/value", name)),
                .. make_value(prefix,"value", 4000+pin*3, ValueSize::FLOAT, ValueDirect::Read(Some(err.into())))
            },
            make_value(prefix, "type", 4100+pin*16, ValueSize::UINT32, ValueDirect::Write), // "Тип датчика"
            make_value(prefix, "point", 4103+pin*16, ValueSize::UINT16, ValueDirect::Write), // "положение десятичной точки"
            make_value(prefix, "Верхняя граница", 4108+pin*16, ValueSize::FLOAT, ValueDirect::Write),
            make_value(prefix, "Нижняя граница", 4110+pin*16, ValueSize::FLOAT, ValueDirect::Write),
            make_value(prefix, "interval", 4113+pin*16, ValueSize::UINT16, ValueDirect::Write),
        ]
    }
}

pub mod owen_digit {
    use super::*;
    pub fn make_klapan(pin: u8, name: &str) -> Vec<Value> {
        let pin = pin as u16 - 1;
        let prefix = name;
        let bitn = pin as u8;
        vec![
            make_value(&prefix, "Режим работы", 272+pin, ValueSize::UINT16, ValueDirect::Write),
            make_value(&prefix, "bit", 470, ValueSize::Bit(bitn), ValueDirect::Write),
        ]
    }
    
    pub fn make_shim(pin: u8, name: &str) -> Vec<Value> {
        let pin = pin as u16 - 1;
        let prefix = name;
        let bitn = pin as u8;
        vec![
            make_value(&prefix, "Режим работы", 272+pin, ValueSize::UINT16, ValueDirect::Write),
            make_value(&prefix, "Период низко-частотного ШИМ", 308+pin, ValueSize::UINT16, ValueDirect::Write),
            make_value(&prefix, "Частота высокочастотного ШИМ", 506+pin, ValueSize::UINT16, ValueDirect::Write),
            
            make_value(&prefix, "Коэффициент заполнения ШИМ", 341+pin, ValueSize::UINT16, ValueDirect::Write),
        ]
    }
    
    pub fn make_counter(pin: u8, name: &str,  value_error: (i32, i32)) -> Vec<Value> {
        let pin = pin as u16 - 1;
        let prefix = name;
        let bitn = pin as u8;
        vec![
            make_value(prefix, "value", 160 +pin*2, ValueSize::UINT32, ValueDirect::Read(Some(value_error.into()))),
            make_value(prefix, "interval", 128 +pin, ValueSize::UINT16, ValueDirect::Write),
            make_value(prefix, "type_input", 64 +pin, ValueSize::UINT16, ValueDirect::Write), // "Дополнительный режим"
            make_value(prefix, "reset_counter", 232 +pin*1, ValueSize::UINT16, ValueDirect::Write), // "Сброс значения счётчика импульсв"
        ]
    }
}
