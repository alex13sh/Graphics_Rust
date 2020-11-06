use std::hash::{Hash, Hasher};

#[derive(Default, Debug, Clone)]
pub struct Value {
    name: String,
    address: u16,
    read_only: bool,
    value: f32, // Cell
}

enum ValueType {
    Analog(f32),
    Digital(bool),
}
