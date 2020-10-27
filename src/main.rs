mod modbus_value;
mod modbus_device;

// mod builder;
// use builder::Builder;
use modbus_value::*;
use modbus_device::*;
fn main() {
    println!("Hello, world!");
    
    let d = ModbusDeviceBuilder::new()
        .name("MyName Divece".into())
        .push_value( ModbusValueBuilder::new()
            .address(12)
            .read_only(true)
            .complete()
        )
        .complete();
        
//     let d = ModbusDevice {
//         name: "MyName Divece".into(),
//         values: vec![
//             ModbusValue {
//                 address: 12,
//                 read_only: true
//             }
//         ]
//     };
        
    dbg!(d);
}
