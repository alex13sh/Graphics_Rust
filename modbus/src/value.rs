// use std::hash::{Hash, Hasher};
pub use super::init::{ValueDirect, ValueSize, Log};
pub use super::init::Value as ValueInit;

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Value {
    name: String,
    address: u16,
    value: Arc<Mutex<(u32, bool)>>,
    // value: [u16, 2]
    pub(super) direct: ValueDirect,
    pub(super) size: ValueSize,
    pub(super) log: Option<Log>,
}

impl Value {
    pub fn new(name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Self {
        Value {
            name: String::from(name),
            address: address,
            direct: direct,
            size: size,
            log: None,
            value: Arc::new(Mutex::new((0, false))),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn address(&self) -> u16 {
        self.address
    }
    pub fn is_read_only(&self) -> bool {
        if let ValueDirect::Read(_) = self.direct {
            true
        } else {
            false 
        }
    }
    pub fn get_error(&self) -> Option<super::ValueError> {
        match self.direct {
        ValueDirect::Read(err) => err,
        ValueDirect::Write => None,
        }
    }
    pub fn is_log(&self) -> bool {
        self.log.is_some()
    }
    pub fn hash(&self) -> String {
        if let Some(ref log) = self.log {
            log.hash.clone()
        } else {
            "".into()
        }
    }
    
    pub fn set_value(&self, value: u32) {
        if let ValueDirect::Write = self.direct {
            // flag set
            let mut v = self.value.lock().unwrap();
            (*v).0 = value;
            (*v).1 = true;
        }
    }
    pub(super) fn clear_flag(&self) {
        (*self.value.lock().unwrap()).1 = false;
    }
    pub(super) fn is_flag(&self) -> bool {
        (*self.value.lock().unwrap()).1
    }
    
    pub(super) fn update_value(&self, value: u32) {
        if value >= std::u32::MAX/2 {
//             dbg!(value);
//             return;
        }
        if value == std::u16::MAX as u32 {
            dbg!(value);
            return;
        }
        (*self.value.lock().unwrap()).0 = value;
    }
    
    pub fn new_value(&self, value: u32) -> Self {
        Self {
            name: self.name.clone(),
            address: self.address,
            value: Arc::new(Mutex::new((value,false))),
            direct: self.direct,
            size: self.size.clone(),
            log: self.log.clone(),
        }
    }
    pub fn value(&self) -> u32 {
        (*self.value.lock().unwrap()).0
    }
    pub fn size(&self) -> u8 {
        self.size.size()
    }
    
    pub fn set_bit(&self, lvl: bool) {
//         self.value.update(|v| {
//             v+1
//         });
        if let ValueSize::Bit(ref num) = self.size {
            let mut v = self.value();
            if lvl {
                v |= 1<<num;
            } else {
                v &= !(1<<num);
            };
            self.set_value(v);
        }
    }
    pub fn get_bit(&self) -> bool {
        if let ValueSize::Bit(ref num) = self.size {
            self.value() & (1<<num) > 0
        } else {false}
    }
    
    pub(crate) fn get_values_bit(&self) -> Vec<Self> {
//         (0..cnt).map(|i| 
        if let ValueSize::BitMap(ref bits) = self.size {
            bits.iter().map(|bit| Self {
                name: bit.name.clone(),
                address: self.address.clone(),
                value: self.value.clone(),
                size: ValueSize::Bit(bit.bit_num),
                direct: self.direct.clone(),
                log: None,
            }).collect()
        } else {Vec::new()}
    }
    pub(crate) fn merge_value(&mut self, val: &Value) {
        self.value = val.value.clone();
    }
}

impl ValueSize {
    pub fn size(&self) -> u8 {
        use ValueSize::*;
        match self {
        INT8 | UINT8 | INT16 | UINT16 | UInt16Map(_) => 1,
        INT32 | UINT32 | FLOAT | FloatMap(_) => 2,
        BitMap(_) | Bit(_) => 1,
        }
    }
}

impl From<ValueInit> for Value {
    fn from(v: ValueInit) -> Self {
        Value {
            name: v.name,
            address: v.address,
            direct: v.direct,
            size: v.size,
            log: v.log,
            value: Arc::new(Mutex::new((0,false))),
        }
    }
}

#[derive(Debug)]
pub enum ValueFloatError {
//     None, // Измерение успешно
    ValueFalse, // 0xF0 -- Значение заведомо неверно
    SensorDisconnect, // 0xF7 -- Датчик отключен
    TemperHight, // 0XF8 -- Велика температура свободных концов ТП
    TemperLow, // 0XF9 -- Мала температура свободных концов ТП
    ValueHigth, // 0xFA -- Измеренное значение слишком велико
    ValueLow, // 0xFB -- Измеренное значение слишком мало
    ShortCircuit, // 0xFC -- Короткое замыкание датчика
//     SensorDisconnect, // 0xFD -- Обрыв датчика
    AdcError, // 0xFE -- Отсутствие связи с АЦП
    RatioError, // 0xFF -- Некорректный калибровочный коэффициент

//     CriticalValue(super::ValueError),
}

impl ValueFloatError {
    fn new(value: u32) -> Option<Self> {
        let lb = value >> 24;
        let res = match lb {
        0xF0 => Some(Self::ValueFalse),
        0xF7 | 0xFD => Some(Self::SensorDisconnect),
        0xF8 => Some(Self::TemperHight),
        0xF9 => Some(Self::TemperLow),
        0xFA => Some(Self::ValueHigth),
        0xFB => Some(Self::ValueLow),
        0xFC => Some(Self::ShortCircuit),
        0xFE => Some(Self::AdcError),
        0xFF => Some(Self::RatioError),
        _ => None,
        };
//         if let Some(ref err) = res {
//             println!("ValueFloatError: {:#X} - {:#X} -- {:?}", value, lb, res);
//         }
        res
    }
}

pub type ValueFloatResult = Result<f32, ValueFloatError>;

pub use std::convert::{TryInto, TryFrom};
impl TryFrom<&Value> for f32 {
    type Error = ValueFloatError;
    fn try_from(val: &Value) -> Result<f32, Self::Error> {
        let res = match val.size {
        ValueSize::FLOAT =>
            if let Some(err) = ValueFloatError::new(val.value()) {
                Err(err)
            } else {Ok(f32::from_bits(val.value()))},
        ValueSize::FloatMap(f) =>
            if let Some(err) = ValueFloatError::new(val.value()) {
                Err(err)
            } else {Ok(f(f32::from_bits(val.value())))},
        ValueSize::UINT32
        | ValueSize::INT32
        | ValueSize::UINT16
        | ValueSize::INT16
        | ValueSize::UINT8
        | ValueSize::INT8 => Ok(val.value() as f32),
        ValueSize::UInt16Map(f) => Ok(f(val.value())),
        _ => Err(ValueFloatError::ValueFalse),
        };
//         if let Ok(v) = res {
//
//         }
        res
    }
}

pub struct ValueArc (Arc<Value>);

// impl TryFrom<ValueArc> for f32 {
// impl TryFrom<Arc<Value>> for f32 {
//     type Error = ();
//     fn try_from(val: Arc<Value>) -> Result<f32, ()> {
//         let v = val.as_ref();
//         f32::try_from(v)
//     }
// }

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone)]
pub struct ModbusValues(HashMap<String, Arc<Value>>);

impl ModbusValues {
    pub fn new() -> Self {
        ModbusValues(HashMap::new())
    }
    pub fn set_value(&self, name: &str, value: u32) -> Arc<Value> {
        let val = self.get(name).unwrap().clone();
        val.update_value(value);
        val
    }
    pub fn get_values_by_name(&self, names: &[&str]) -> ModbusValues {
        ModbusValues (
            names.iter().filter_map(
                |&name| self.get(name).map(|v|
                    (String::from(name), Arc::clone(v))
                )
            ).collect()
        )
    }
    pub fn get_values_by_name_starts(&self, names: &[&str]) -> ModbusValues {
        ModbusValues (
            self.0.iter().filter(|(k, v)| {
                names.iter().any(|&name| k.starts_with(name))
            }).map(|(k,v)|(k.clone(), v.clone())).collect()
        )
    }
}

impl From<Vec<ValueInit>> for ModbusValues {
    fn from(values: Vec<ValueInit>) -> Self {
        ModbusValues(
            values.into_iter()
                .map(|v| (v.name.clone(), Arc::new(Value::from(v))))
                .collect()
        )
    }
}

impl From<HashMap<String, Arc<Value>>> for ModbusValues {
    fn from(values: HashMap<String, Arc<Value>>) -> Self {
        ModbusValues(values)
    }
}


impl Deref for ModbusValues {
    type Target = HashMap<String, Arc<Value>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ModbusValues {
//     type Target = HashMap<String, Arc<Value>>;
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for ValueArc {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// try into bool, i32, f32

#[test]
fn test_value_ops_bit() {
    let v = Value::from(ValueInit{
        name: "Name_1".into(),
        address: 1,
        direct: ValueDirect::Write,
        size: ValueSize::BitMap(vec![]),
    });
    v.set_bit(1, true);
    assert_eq!(v.value.get(), 2);
    v.set_bit(4, true);
    assert_eq!(v.value.get(), 18);
    assert_eq!(v.get_bit(3), false);
    assert_eq!(v.get_bit(4), true);
}

#[test]
fn test_value_into_f32() {
    let v = Value::from(ValueInit{
        name: "Name_1".into(),
        address: 1,
        direct: ValueDirect::Write,
        size: ValueSize::FLOAT,
    });
    v.value.set(u32::from_le_bytes([0x00,0x00,0x20,0x3E]));
    let f: f32 = (&v).try_into().unwrap();
    assert_eq!(f, 0.15625);
    let f = f32::from_le_bytes([0x00,0x00,0x20,0x3E]);
    assert_eq!(f, 0.15625);
}
