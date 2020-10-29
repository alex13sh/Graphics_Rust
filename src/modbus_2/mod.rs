mod value;
mod sensor;
mod device;

pub use value::*;
pub use sensor::*;
pub use device::*;


// #[test]
pub fn tst() {

    let d = Device {
        name: "Device Name".into(),
        sensors: vec![
            Sensor {
                name: "Device_1/Sensor_1".into(),
                values: vec![
                    Value {
                        name: "Device_1/Sensor_1/Value_1".into()
                    },
                    Value {
                        name: "Device_1/Sensor_1/Value_2".into()
                    },
                ]
            },
            Sensor {
                name: "Device_1/Sensor_2".into(),
                values: vec![
                    Value {
                        name: "Device_1/Sensor_2/Value_1".into()
                    },
                    Value {
                        name: "Device_1/Sensor_2/Value_2".into()
                    },
                ]
            }
        ]
    };
    
    dbg!(d);
}

// #[test]
// #[cfg(false)]
/*
fn tst() {
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
