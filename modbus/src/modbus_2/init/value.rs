
#[derive(Debug)]
pub struct Value {
    pub name: String,
    pub address: u16,
    pub direct: ValueDirect,
    pub size: ValueSize,
}

#[derive(Debug)]
pub enum ValueDirect {
    Read,
    Write
}

impl Default for ValueDirect {
    fn default() -> Self {
        ValueDirect::Write
    }
}

#[derive(Debug)]
pub enum ValueSize {
    INT8,
    UINT8,
    INT16,
    UINT16,
    INT32,
    UINT32,
    FLOAT,
    BitMap
}

impl Default for ValueSize {
    fn default() -> Self {
        ValueSize::FLOAT
    }
}
