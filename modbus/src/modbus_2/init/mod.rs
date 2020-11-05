mod sensor;
mod device;

pub use sensor::*;
pub use device::*;


// #[test]
pub fn tst() {
    use SensorAnalogType::*;

    let d = Device {
        name: "Input Analog".into(),
        device_type: DeviceType::OwenAnalog,
        sensors: vec![
            Sensor {
                name: "Температура Ротора".into(),
                sensor_type: SensorType::Analog(Amper_4_20),
                pin: 1,
                interval: 800,
                value_error: (60, 90).into(),
            },
            Sensor {
                name: "Давление -1_1 V".into(),
                sensor_type: SensorType::Davl(Volt_1),
                pin: 2,
                interval: 800,
                value_error: (0.1, 0.5).into(),
            },
            Sensor {
                name: "Вибрация 4_20 A".into(),
                sensor_type: SensorType::Vibra(Amper_4_20),
                pin: 3,
                interval: 600,
                value_error: (3, 5).into(),
            }
        ]
    };
    
    dbg!(d);
}
