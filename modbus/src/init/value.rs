#![allow(non_camel_case_types, dead_code)]

#[derive(Debug, Clone)]
pub struct Value {
    pub name: String,
    pub address: u16,
    pub direct: ValueDirect,
    pub size: ValueSize,
    pub log: Option<Log>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueDirect {
    Read {
        max: Option<ValueError>,
        min: Option<ValueError>,
    }, // (interval)
    Write
}

impl Default for ValueDirect {
    fn default() -> Self {
        ValueDirect::Write
    }
}

impl ValueDirect {
    pub fn read() -> Self {
        ValueDirect::Read {
            min: None,
            max: None,
        }
    }
    pub fn err_min(self, min: ValueError) -> Self {
        if let ValueDirect::Read {max, ..} = self {
            ValueDirect::Read {
                max: max,
                min: Some(min),
            }
        } else {
            self
        }
    }
    pub fn err_max(self, max: ValueError) -> Self {
        if let ValueDirect::Read {min, ..} = self {
            ValueDirect::Read {
                max: Some(max),
                min: min,
            }
        } else {
            self
        }
    }
    pub fn err_min_max(self, min: ValueError, max: ValueError) -> Self {
        if let ValueDirect::Read {..} = self {
            ValueDirect::Read {
                max: Some(max),
                min: Some(min),
            }
        } else {
            self
        }
    }
}

#[test]
fn test_read_error_init() {
    let dir = ValueDirect::read().err_max((8,10).into());
    assert_eq!(dir, ValueDirect::Read {
        min: None,
        max: Some( ValueError {yellow: 8.0, red: 10.0})
    });
}

#[derive(Debug, Clone)]
pub enum ValueSize {
    INT8,
    UINT8,
    INT16,
    UINT16,
    UInt16Map(fn(u32) -> f32),
    INT32,
    UINT32,
    FLOAT,
    FloatMap(fn(f32) -> f32),
    BitMap (Vec<ValueBit>),
    Bit(u8), // Номер бита
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

#[derive(Debug, Clone)]
pub struct Log {
    pub hash: String,       // HEX
    pub full_name: String, // DeviceName/SensorName/ValueName
}

impl Log {
    pub fn hash(hash: &str) -> Option<Log> {
        Some(Log {
            hash: hash.into(),
            full_name: "".into(),
        })
    }
}
