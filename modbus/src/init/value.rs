#![allow(non_camel_case_types, dead_code)]

#[derive(Debug, Clone)]
pub struct Value {
    pub name: ValueID,
    pub suffix_name: Option<String>,
    pub address: u16,
    pub direct: ValueDirect,
    pub size: ValueSize,
    pub log: Option<Log>,
}

impl Value {
    pub fn new(address: u16, name: &str) -> Self {
        Value {
            name: name.into(),
            suffix_name: None,
            address: address,
            direct: ValueDirect::Write,
            size: ValueSize::UINT16,
            log: None,
        }
    }
    pub fn make_value(name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Self {
        Value {
            name: name.into(),
            suffix_name: None,
            address: address,
            direct: direct,
            size: size,
            log: None,
        }
    }
    pub fn with_sensor(mut self, sensor_name: &str) -> Self {
        self.name.sensor_name = sensor_name.into();
        self
    }
    pub fn with_log(mut self, log: Log) -> Self {
        self.log = Some(log);
        self
    }
    pub fn with_suffix(mut self, suffix_name: &str) -> Self {
        self.suffix_name = Some(suffix_name.into());
        self
    }
    pub fn direct(mut self, direct: ValueDirect) -> Self {
        self.direct = direct;
        self
    }
    pub fn size(mut self, size: ValueSize) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValueID {
    pub device_id: u16,
    pub device_name: String,
    pub sensor_name: String,
    pub value_name: String,
}

impl From<&str> for ValueID {
    fn from(name: &str) -> Self {
        ValueID::value(name)
    }
}

impl std::fmt::Display for ValueID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}) {}/{}/{}", self.device_id, self.device_name, self.sensor_name, self.value_name)
    }
}

impl ValueID {
    fn value(name: &str) -> Self {
        Self {
            value_name: name.into(),
            .. Default::default()
        }
    }
    pub fn sensor_bit(name: &str) -> Self {
        Self {
            sensor_name: name.into(),
            value_name: "bit".into(),
            .. Default::default()
        }
    }
    pub fn sensor_value(name: &str) -> Self {
        Self {
            sensor_name: name.into(),
            value_name: "value".into(),
            .. Default::default()
        }
    }
    pub fn sensor_value_name(&self) -> String {
        format!("{}/{}", self.sensor_name, self.value_name)
    }
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
    UInt16Dot(u8), // Точка смещения
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
    pub device_name: String,
    pub sensor_name: String,
    pub value_name: String,
//     pub value_address: u16,
    pub device_id: u16,
}

impl Log {
    pub fn value(self,/* adr: u16,*/ name: &str) -> Self {
        Self {
//             address: adr,
            value_name: name.into(),
            .. self
        }
    }
    pub fn sensor(name: &str) -> Self {
        Self {
            device_id: 0,
            device_name: String::new(),
            sensor_name: name.into(),
            value_name: String::new(),
        }
    }
    pub fn device(self, id:u16, name: &str) -> Self {
        Self {
            device_id: id,
            device_name: name.into(),
            .. self
        }
    }
    pub fn print_full_name(&self) -> String {
        format!("{}/{}/{}", self.device_name, self.sensor_name, self.value_name)
    }
}
