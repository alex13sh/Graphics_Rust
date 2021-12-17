// use std::hash::{Hash, Hasher};
pub use super::init::{ValueDirect, ValueSize, Log};
pub use super::init::{self, Value as ValueInit};

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Value {
    id: ValueID,
    suffix_name: Option<String>,
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
            id: init::ValueID::from(name).into(),
            suffix_name: None,
            address: address,
            direct: direct,
            size: size,
            log: None,
            value: Arc::new(Mutex::new((0, false))),
        }
    }
    pub fn name(&self) -> &String {
        &self.id.sensor_name
    }
    pub fn full_name(&self) -> String {
        self.id.to_string()
    }
    pub fn id(&self) -> &ValueID {
        &self.id
    }
    
    pub fn address(&self) -> u16 {
        self.address
    }
    pub fn suffix_name(&self) -> Option<&String> {
        Some(self.suffix_name.as_ref()?)
    }
    pub fn is_read_only(&self) -> bool {
        if let ValueDirect::Read{..} = self.direct {
            true
        } else {
            false 
        }
    }
    pub fn get_error_max(&self) -> Option<super::ValueError> {
        match self.direct {
        ValueDirect::Read{max, ..} => max,
        ValueDirect::Write => None,
        }
    }
    pub fn get_error_min_max(&self) -> (Option<super::ValueError>, Option<super::ValueError>) {
        match self.direct {
        ValueDirect::Read{min, max} => (min, max),
        ValueDirect::Write => (None, None),
        }
    }
    pub fn is_error(&self) -> bool {
        use std::convert::TryFrom;
        let v = f32::try_from(self);
        if let Ok(v) = v {
            match self.get_error_min_max() {
            (None, Some(max)) => max.red < v,
            (Some(min), Some(max)) => max.red < v || min.red > v,
            _ => false,
            }
        } else {true}
    }

    pub fn is_log(&self) -> bool {
        self.log.is_some()
    }
    pub fn hash(&self) -> String {
        if let Some(ref log) = self.log {
            log.print_full_name()
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
    pub/*(super)*/ fn is_flag(&self) -> bool {
        (*self.value.lock().unwrap()).1
    }
    
    pub(super) fn update_value(&self, value: u32) {
//         if value >= std::u32::MAX/2 {
// //             dbg!(value);
// //             return;
//         }
//         if value == std::u16::MAX as u32 {
// //             dbg!(value);
//             return;
//         }
        let mut l = self.value.lock().unwrap();
        if !(self.is_log() && l.1 == true) {
            (*l).0 = value;
        }
    }
    
//     pub fn new_value(&self, value: u32) -> Self {
//         Self {
//             value: Arc::new(Mutex::new((value,false))),
//             .. self.clone()
//         }
//     }
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
                id: init::ValueID::sensor_bit(&bit.name).into(),
                suffix_name: None,
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

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct ValueID {
    pub device_id: u16,
    pub device_name: String,
    pub sensor_name: String,
    pub value_name: String,
}

impl From<init::ValueID> for ValueID {
    fn from(v: init::ValueID) -> Self {
        ValueID {
            device_id: v.device_id.unwrap_or(0),
            device_name: v.device_name.unwrap_or("".into()),
            sensor_name: v.sensor_name.unwrap_or("".into()),
            value_name: v.value_name.unwrap_or("value".into()),
        }
    }
}

impl std::fmt::Display for ValueID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}) {}/{}/{}", self.device_id, self.device_name, self.sensor_name, self.value_name)
    }
}

impl ValueID {
    pub fn sensor_value_name(&self) -> String {
        format!("{}/{}", self.sensor_name, self.value_name)
    }
}

impl PartialEq<init::ValueID> for ValueID {
    fn eq(&self, other: &init::ValueID) -> bool {
        other.device_id.map_or(true, |id| self.device_id == id) && 
        other.device_name.as_ref().map_or(true, |name| &self.device_name == name) && 
        other.sensor_name.as_ref().map_or(true, |name| &self.sensor_name == name) && 
        other.value_name.as_ref().map_or(true, |name| &self.value_name == name)
    }
}

impl ValueSize {
    pub fn size(&self) -> u8 {
        use ValueSize::*;
        match self {
        INT8 | UINT8 | INT16 | UINT16 | UInt16Dot(_) | UInt16Map(_) => 1,
        INT32 | UINT32 | FLOAT | FloatMap(_) => 2,
        BitMap(_) | Bit(_) => 1,
        }
    }
}

impl From<ValueInit> for Value {
    fn from(v: ValueInit) -> Self {
        Value {
            id: v.name.into(),
            suffix_name: v.suffix_name,
            address: v.address,
            direct: v.direct,
            size: v.size,
            log: v.log,
            value: Arc::new(Mutex::new((0,false))),
        }
    }
}

#[cfg(test)]
fn test_value_init() -> Value {
    Value::from(ValueInit{
        name: "Name_1".into(),
        suffix_name: Some("".into()),
        address: 1,
        direct: ValueDirect::Write,
        size: ValueSize::FLOAT,
        log: None,
    })
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
    fn new_u16(value: u32) -> Option<Self> {
        if value == 32768 {
            Some(Self::SensorDisconnect)
        } else {
            None
        }
    }
}

pub type ValueFloatResult = Result<f32, ValueFloatError>;

pub use std::convert::{TryInto, TryFrom};
impl TryFrom<&Value> for f32 {
    type Error = ValueFloatError;
    fn try_from(val: &Value) -> Result<f32, Self::Error> {
        const fdot: [f32; 4] = [1_f32, 10_f32, 100_f32, 1_000_f32];
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
        ValueSize::UInt16Map(f) =>
            if let Some(err) = ValueFloatError::new_u16(val.value()) {
                Err(err)
            } else {Ok(f(val.value()))},
        ValueSize::UInt16Dot(dot) =>
            if let Some(err) = ValueFloatError::new_u16(val.value()) {
                Err(err)
            } else {Ok(val.value() as f32 / fdot[dot as usize] )},
        ValueSize::Bit(_pin) => Ok(if val.get_bit() {1.0} else {0.0}),
        _ => Err(ValueFloatError::ValueFalse),
        };
//         if let Ok(v) = res {
//
//         }
        res
    }
}

pub type ValueArc = Arc<Value>;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone)]
pub struct ModbusValues(HashMap<ValueID, Arc<Value>>);

impl ModbusValues {
    pub fn new() -> Self {
        ModbusValues(HashMap::new())
    }

    pub fn get_value_arc(&self, name: &str) -> Option<ValueArc> {
        let s_value = init::ValueID::sensor_value(name).into();
        let s_bit = init::ValueID::sensor_bit(name).into();
        let v = self.get(&s_value)
            .or_else(||self.get(&s_bit))?;
        Some(v.clone())
    }

    pub fn set_value(&self, name: &str, value: u32) -> ValueArc {
        let val = self.get_value_arc(name).unwrap();
        val.update_value(value);
        val
    }
    pub fn get_values_by_name(&self, names: &[&str]) -> ModbusValues {
        ModbusValues (
            names.iter().filter_map(
                |&name| self.get_value_arc(name).map(|v|
                    (v.id.clone(), v)
                )
            ).collect()
        )
    }
    
    pub fn get_values_by_id(&self, f: impl Fn(&ValueID) -> bool) -> ModbusValues {
        ModbusValues (
            self.0.iter().filter(|(id, v)| f(id))
                .map(|(id, v)| (id.clone(), v.clone()))
                .collect()
        )
    }
    
    pub fn iter_values(&self) -> impl Iterator<Item=(u16, u32, &ValueID)> + '_ {
        self.0.iter().map(|(k, v)| (
                v.address(), //((v.address()/256) as u8, (v.address()%256) as u8),
                v.value(), k
            ))
    }
}

impl IntoIterator for ModbusValues {
    type Item = <HashMap<ValueID, Arc<Value>> as IntoIterator>::Item;
    type IntoIter = <HashMap<ValueID, Arc<Value>> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::ops::Add<ModbusValues> for ModbusValues {
    type Output = ModbusValues;
    fn add(self, other: Self) -> Self {
        ModbusValues (
            self.0.into_iter()
                .chain(other.0.into_iter())
                .collect()
        )
    }
}

impl <'a> std::ops::Add<&'a ModbusValues> for &'a ModbusValues {
    type Output = ModbusValues;
    fn add(self, other: Self) -> Self::Output {
        ModbusValues (
            self.0.iter()
                .chain(other.0.iter())
                .map(|(name, v)| (name.clone(), v.clone()))
                .collect()
        )
    }
}

impl From<Vec<ValueInit>> for ModbusValues {
    fn from(values: Vec<ValueInit>) -> Self {
        ModbusValues(
            values.into_iter()
                .map(|v| (v.name.clone().into(), Arc::new(Value::from(v))))
                .collect()
        )
    }
}

impl From<HashMap<ValueID, Arc<Value>>> for ModbusValues {
    fn from(values: HashMap<ValueID, Arc<Value>>) -> Self {
        ModbusValues(values)
    }
}

impl From<ModbusValues> for Vec<ValueArc> {
    fn from(values: ModbusValues) -> Self {
        values.0.into_iter().map(|v| v.1.clone()).collect()
    }
}
impl From<Vec<ValueArc>> for ModbusValues {
    fn from(values: Vec<ValueArc>) -> Self {
        values.into_iter().collect()
    }
}

impl std::iter::FromIterator<Arc<Value>> for ModbusValues {
    fn from_iter<I: IntoIterator<Item=Arc<Value>>>(iter: I) -> Self {
        let mut c = ModbusValues::new();
        
        for i in iter {
            c.insert(i.id().clone(), i);
        }

        c
    }
}


impl Deref for ModbusValues {
    type Target = HashMap<ValueID, Arc<Value>>;
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


// try into bool, i32, f32

// #[test]
// fn test_value_ops_bit() {
//     let v = Value::from(ValueInit{
//         name: "Name_1".into(),
//         address: 1,
//         direct: ValueDirect::Write,
//         size: ValueSize::BitMap(vec![]),
//         log: None,
//     });
//     v.set_bit(1, true);
//     assert_eq!(v.value.get(), 2);
//     v.set_bit(4, true);
//     assert_eq!(v.value.get(), 18);
//     assert_eq!(v.get_bit(3), false);
//     assert_eq!(v.get_bit(4), true);
// }

#[test]
fn test_value_into_f32() {
    let v = test_value_init();
    *v.value.lock().unwrap() = (u32::from_le_bytes([0x00,0x00,0x20,0x3E]), false);
    let f: f32 = (&v).try_into().unwrap();
    assert_eq!(f, 0.15625);
    let f = f32::from_le_bytes([0x00,0x00,0x20,0x3E]);
    assert_eq!(f, 0.15625);
}
