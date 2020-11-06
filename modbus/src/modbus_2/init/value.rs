
#[derive(Debug)]
pub struct Value {
    pub name: String,
    pub address: u16,
    pub value_type: ValueType,
}

#[derive(Debug)]
pub enum ValueType {
    AnalogInput {
        interval: u32,
    },
    DigitalInput {
        interval: u32,
    },
    DigitalOutput,
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::AnalogInput{interval: 1000}
    }
}
