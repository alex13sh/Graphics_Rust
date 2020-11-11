use std::hash::{Hash, Hasher};
pub use super::init::{ValueDirect, ValueSize};

#[derive(Debug, Default)]
pub struct Value {
    name: String,
    address: u16,
    value: u32, // Cell
    // value: [u16, 2]
    direct: ValueDirect,
    size: ValueSize,
}

impl Value {
    pub fn new(name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Self {
        Value {
            name: String::from(name),
            address: address,
            direct: direct,
            size: size,
            value: 0,
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl ValueSize {
    pub fn size(&self) -> u8 {
        use ValueSize::*;
        match self {
        INT8 | UINT8 | INT16 | UINT16 => 1,
        INT32 | UINT32 | FLOAT => 2,
        BitMap => 1,
        }
    }
}

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ModbusValues(HashMap<String, Arc<Value>>);

impl ModbusValues {
    pub fn new() -> Self {
        ModbusValues(HashMap::new())
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

// try into bool, i32, f32
