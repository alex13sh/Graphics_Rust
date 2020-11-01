use super::{Builder, Name};
use std::hash::{Hash, Hasher};

#[derive(Default, Debug, Clone)]
pub struct Value {
    name: String,
    address: u16,
//     hash: String,
//     type: Type,
    read_only: bool,
    sensor_name: Option<Box<dyn Name>>
}

enum Type {

}

// impl MudbusValue {
//     
// }

impl Name for Value {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub type ValueBuilder = Builder<Value>;
impl Builder<Value> {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.obj.name = name.into();
        self
    }
    pub fn address(mut self, value: u16) -> Self {
        self.obj.address = value;
        self
    }
    pub fn read_only(mut self, value: bool) -> Self {
        self.obj.read_only = value;
        self
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
//         self..hash(state);
    }
}
