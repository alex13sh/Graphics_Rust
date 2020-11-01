mod value;
mod sensor;
mod device;

#[path = "../builder.rs"]
mod builder;
use builder::Builder;
pub use value::*;
pub use sensor::*;
pub use device::*;

trait Name {
    fn name(&self) -> String;
}

// #[test]
pub fn tst() {

    
}



// #[test]
// #[cfg(false)]
/*
fn tst_main() {
    use super as modbus;
println!("Hello, world!");
    
    let d = modbus::DeviceBuilder::new()
        .name("MyName Divece".into())
        .push_sensor_builder (
            modbus::SensorBuilder::new()
            .name("Sensor 1".into())
            .push_value(
                modbus::ValueBuilder::new()
                .address(12)
                .read_only(true)
                .complete()
            )
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
}*/
