mod value;
mod sensor;
mod device;

pub use value::*;
pub use sensor::*;
pub use device::*;


// #[test]
pub fn tst() {
    use SensorAnalogType::*;
    use ValueGroup::*;

    let d = vec![
    Device {
        name: "Input Analog".into(),
        device_type: DeviceType::OwenAnalog,
        sensors: Some(vec![
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
            },
            Sensor {
                name: "Температура Статора".into(),
                sensor_type: SensorType::Analog(Pt_100),
                pin: 4,
                interval: 800,
                value_error: (60, 85).into(),
            },
            Sensor {
                name: "Температура Пер.Под.".into(),
                sensor_type: SensorType::Analog(Pt_100),
                pin: 5,
                interval: 800,
                value_error: (60, 80).into(),
            },
            Sensor {
                name: "Температура Зад.Под.".into(),
                sensor_type: SensorType::Analog(Pt_100),
                pin: 6,
                interval: 800,
                value_error: (60, 80).into(),
            },
        ]),
        values: None,
    },
    Device {
        name: "Input/Output Digit".into(),
        device_type: DeviceType::OwenDigitalIO,
        sensors: Some(vec![
            Sensor {
                name: "Скоростной счётчик импульсов".into(),
                sensor_type: SensorType::Counter(0),
                pin: 0,
                interval: 2,
                value_error: (333, 433).into(),
            },
            GroupPin {
                name: "Клапан 24В".into(),
                group_type: GroupValueType::DigitalOutput(false),
                pin: 1,
            },
            GroupPin {
                name: "Клапан 2".into(),
                group_type: GroupValueType::DigitalOutput(false),
                pin: 2,
            },
            GroupPin {
                name: "Насос".into(),
                group_type: GroupValueType::DigitalOutput(false),
                pin: 3,
            },
        ]),
        values: None,
    }
    ];
    
    dbg!(d);
}
