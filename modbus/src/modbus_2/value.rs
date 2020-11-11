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

// try into bool, i32, f32
