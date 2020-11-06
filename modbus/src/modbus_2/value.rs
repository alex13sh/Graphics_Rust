use std::hash::{Hash, Hasher};
use super::init::{ValueType};

#[derive(Debug, Default)]
pub struct Value {
    name: String,
    address: u16,
    value: u32, // Cell
    // value: [u16, 2]
    value_type: ValueType,
}

impl Value {
    pub fn new(name: &str, address: u16, typ: ValueType) -> Self {
        Value {
            name: String::from(name),
            address: address,
            value_type: typ,
            value: 0,
        }
    }
}

// try into bool, i32, f32
