#[path = "./builder.rs"]
mod builder;
use builder::Builder;
use std::hash::{Hash, Hasher};

#[derive(Default, Debug)]
pub struct ModbusValue {
    address: u16,
//     hash: String,
//     type: Type,
    read_only: bool,
}

enum Type {

}

// impl MudbusValue {
//     
// }

pub type BuilderModbusValue = Builder<ModbusValue>;
impl Builder<ModbusValue> {
    pub fn address(mut self, value: u16) -> Self {
        self.obj.address = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.obj.read_only = value;
        self
    }
}

impl Hash for ModbusValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
//         self..hash(state);
    }
}
