#![allow(non_camel_case_types, dead_code)]

#[derive(Debug)]
pub struct Value {
    pub name: String,
    pub address: u16,
    pub direct: ValueDirect,
    pub size: ValueSize,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ValueError {
    pub yellow: f32,
    pub red: f32
}

impl From<(f32, f32)> for ValueError {
    fn from((y, r): (f32, f32)) -> Self {
        Self {yellow: y, red: r}
    }
}
impl From<(i32, i32)> for ValueError {
    fn from((y, r): (i32, i32)) -> Self {
        Self {yellow: y as f32, red: r as f32}
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ValueDirect {
    Read(Option<ValueError>), // (interval)
    Write
}

impl Default for ValueDirect {
    fn default() -> Self {
        ValueDirect::Write
    }
}

#[derive(Debug, Clone)]
pub enum ValueSize {
    INT8,
    UINT8,
    INT16,
    UINT16,
    INT32,
    UINT32,
    FLOAT,
    BitMap (Vec<ValueBit>),
    // UINT16_FLOAT(u8 offset),
    // INT16_FLOAT(u8 offset),
}

impl Default for ValueSize {
    fn default() -> Self {
        ValueSize::FLOAT
    }
}

#[derive(Debug, Clone)]
pub struct ValueBit {
    pub name: String,
    pub bit_num: u8,
    pub bit_size: u8,
}
