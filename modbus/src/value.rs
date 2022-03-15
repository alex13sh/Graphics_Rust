pub mod modbusvalues;
pub use modbusvalues::*;
mod tests;
mod types;
pub use types::*;

// use std::hash::{Hash, Hasher};
pub use super::init::{ValueDirect, ValueSize, Log};
pub use super::init::{self, Value as ValueInit};

use std::sync::{Arc, Mutex};
pub type ValueArc = Arc<Value>;

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
                id: ValueID{ 
                    sensor_name: bit.name.clone(),
                    value_name: "bit".into(),
                    .. self.id.clone()
                },
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

impl Value {
    pub fn value_as_f32(&self) -> f32 {
        match f32::try_from(self) {
        Ok(res) => res,
        Err(err) => panic!("Value err: {:?}; for value: {:?}", err, self.id),
        }
    }
    pub fn try_value_as_f32(&self) -> Option<f32> {
        f32::try_from(self).ok()
    }
}

impl From<Vec<ValueInit>> for ModbusValues {
    fn from(values: Vec<ValueInit>) -> Self {
        values.into_iter()
            .map(|v| ValueArc::new(v.into()))
            .collect()
    }
}
