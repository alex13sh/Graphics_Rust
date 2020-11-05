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
