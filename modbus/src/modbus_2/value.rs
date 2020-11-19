// use std::hash::{Hash, Hasher};
pub use super::init::{ValueDirect, ValueSize};
pub use super::init::Value as ValueInit;

use std::cell::Cell;

#[derive(Debug)]
pub struct Value {
    name: String,
    address: u16,
    value: Cell<u32>, // Cell
    // value: [u16, 2]
    pub(super) direct: ValueDirect,
    pub(super) size: ValueSize,
}

impl Value {
    pub fn new(name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Self {
        Value {
            name: String::from(name),
            address: address,
            direct: direct,
            size: size,
            value: Cell::new(0),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn address(&self) -> u16 {
        self.address
    }
    pub fn is_read_only(&self) -> bool {
        if let ValueDirect::Read = self.direct {
            true
        } else {
            false 
        }
    }
    pub(super) fn update_value(&self, value: u32) {
        self.value.set(value);
    }
    
    pub fn new_value(&self, value: u32) -> Self {
        Self {
            name: self.name.clone(),
            address: self.address,
            value: Cell::new(value),
            direct: self.direct,
            size: self.size,
        }
    }
    pub fn value(&self) -> u32 {
        self.value.get()
    }
    
    pub fn set_bit(&self, num: u8, lvl: bool) {
//         self.value.update(|v| {
//             v+1
//         });
        let mut v = self.value.get();
        if lvl {
            v |= 1<<num;
        } else {
            v &= !(1<<num);
        };
        self.value.set(v);
    }
    pub fn get_bit(&self, num: u8) -> bool {
        self.value.get() & (1<<num) > 0
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

impl From<ValueInit> for Value {
    fn from(v: ValueInit) -> Self {
        Value {
            name: v.name,
            address: v.address,
            direct: v.direct,
            size: v.size,
            value: Cell::new(0),
        }
    }
}

use std::convert::TryInto;
impl TryInto<f32> for Value {
    type Error = ();
    fn try_into(self) -> Result<f32, ()> {
        match self.size {
        ValueSize::FLOAT => Ok(f32::from_bits(self.value.get())),
        ValueSize::UINT32
        | ValueSize::INT32
        | ValueSize::UINT16
        | ValueSize::INT16
        | ValueSize::UINT8
        | ValueSize::INT8 => Ok(self.value.get() as f32),
        _ => Err(()),
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

#[test]
fn test_value_ops_bit() {
    let v = Value::from(ValueInit{
        name: "Name_1".into(),
        address: 1,
        direct: ValueDirect::Write,
        size: ValueSize::BitMap,
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
    let f: f32 = v.try_into().unwrap();
    assert_eq!(f, 0.15625);
    let f = f32::from_le_bytes([0x00,0x00,0x20,0x3E]);
    assert_eq!(f, 0.15625);
}
