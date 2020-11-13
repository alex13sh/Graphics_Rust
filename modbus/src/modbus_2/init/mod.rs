mod value;
mod sensor;
mod device;

pub use value::*;
pub use sensor::*;
pub use device::*;


// #[test]
pub fn tst() {
    let d = init_devices();
    dbg!(d);
}

pub fn init_devices() -> Vec<Device> {
    use SensorAnalogType::*;
    use ValueGroup::*;

    let add_simple_invertor_value = |name: &str, p: u16, adr: u16| Value {
        name: name.into(),
        address: p*256+adr,
        direct: ValueDirect::Write,
        size: ValueSize::UINT16,
    };
    
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
    },
    Device {
        name: "Invertor".into(),
        device_type: DeviceType::Invertor {
            functions: vec![
                InvertorFunc::DigitalOutput(0, 2), // Заданная частота достигнута
                InvertorFunc::DigitalOutput(0, 13), // Предупреждение о перегреве радиатора
            ]
        },
        sensors: Some(vec![
//             Group {
//                 name: "".into(),
//                 values: vec![],
//             },
        ]),
        values: Some(vec![
            add_simple_invertor_value("Сброс параметров",  0, 2), // 0 - 10
            
            add_simple_invertor_value("Режим управления",  0, 10), // 0 - 2
            add_simple_invertor_value("Метод управления скоростью",  0, 11), // 0 - 3
            add_simple_invertor_value("Режим работы привода",   0, 16), // 0 - 1
            add_simple_invertor_value("Несущая частота ШИМ",    0, 17), // Таблица преобразований
            add_simple_invertor_value("Управление направлением вращения двигателя",  0, 23), // 0 - 2
            add_simple_invertor_value("Сбособ остановки",   0, 22), // 0 - 1
            
            add_simple_invertor_value("Источник задания частоты",           0, 20), // 0 - 8 // 8 - Плата
            add_simple_invertor_value("Источник команд управления",         0, 21), // 0 - 5 // 5 - Плата
            add_simple_invertor_value("Источник задания частоты (HAND)",    0, 30), // 0 - 8 // 8 - Плата
            
            add_simple_invertor_value("Максимальная выходная частота",      1, 0), // 50.0 - 600.0
            add_simple_invertor_value("Номинальная частота двигателя",      1, 1), // 0.0 - 600.0
            add_simple_invertor_value("Номинальное напряжение двигателя",   1, 2), // 0 - 255.0
            add_simple_invertor_value("Сбособ остановки",  0, 22), // 0 - 1
            
            add_simple_invertor_value("Стартовая частота",  1, 9), // 0.00 - 600.00
            add_simple_invertor_value("Верхнее ограничение выходной частота",  1, 10), // 0.00 - 600.00
            add_simple_invertor_value( "Нижнее ограничение выходной частота",  1, 11), // 0.00 - 600.00
            
            // 1.12 - 1.21 -- Временные параметры
            
            // 1.28 - 1.33 -- Частота пропуска (1, 2, 3)
            
            add_simple_invertor_value("Выбор режима разгона/замедления",  1, 44), // 0 - 4
            
            // 2.1 - 2.8, 2.26 - 2.31 -- Дискретные входы // Значения 0 - 53
            // add_simple_invertor_value("Выбор состояния для дискретных входов",  2, 12), // BitMap 16 bit
            Value {
                name: "Выбор состояния для дискретных входов".into(),
                address: 2*256+12,
                direct: ValueDirect::Write,
                size: ValueSize::BitMap, // 16 bit
            },
            add_simple_invertor_value("Скорость изменения частоты командами Up/Down",  2, 10), // 0.01 - 1.00
            
            // 2.36 - 2.46 -- Цифровые выходы // Значения 0 - 51
            // f-2 -- Заданная частота достигнута
            // f-13 -- Предупреждение о перегреве радиатора
            // f-27-28 -- Выходной ток выше или ниже p-2-33
            // f-35-39 -- Индикация ошибок p-6-23-26
            
            Value {
                name: "Выбор неактивного состояния для дискретных выходов".into(),
                address: 2*256+18,
                direct: ValueDirect::Write,
                size: ValueSize::BitMap, // 16 bit
            },
        ]),
    }
    ];
    return d;
}
