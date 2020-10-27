mod modbus_value;
mod modbus_device;

mod builder;
// use builder::Builder;
use modbus_value::*;
use modbus_device::*;
fn main() {
    println!("Hello, world!");
    
    let d = BuilderModbusDevice::new()
        .name("MyName Divece".into())
        .push_value(
            BuilderModbusValue::new()
            .address(12)
            .read_only(true)
            .complete()
        )
        .complete();
        
    dbg!(d);
}
