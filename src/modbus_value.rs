#[path = "./builder.rs"]
mod builder;
pub use builder::Builder;

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
