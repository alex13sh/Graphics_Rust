mod modbus_value;
// pub mod builder;

// use builder::Builder;
use modbus_value::{ModbusValue, Builder};

fn main() {
    println!("Hello, world!");
    let v = Builder::<ModbusValue>::new()
        .address(12)
        .read_only(true)
        .complete();
    dbg!(v);
}
