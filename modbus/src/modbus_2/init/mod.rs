#![allow(dead_code)]

mod value;
mod sensor;
mod device;

pub use value::*;
pub use sensor::*;
pub use device::*;


// #[test]
pub(crate) fn tst() {
    let d = init_devices();
    dbg!(d);
}

pub(crate) fn init_devices() -> Vec<Device> {    
    vec![
    make_owen_analog("192.168.1.5".into()),
    make_io_digit("192.168.1.3".into()),
    make_invertor("192.168.1.5".into()),
    ]
}

pub fn make_owen_analog(ip_address: String) -> Device {
    use SensorAnalogType::*;
    use ValueGroup::*;
    
    Device {
        name: "Input Analog".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIP(ip_address),
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
    }
}

pub fn make_io_digit(ip_address: String) -> Device {
    use SensorAnalogType::*;
    use ValueGroup::*;
    Device {
        name: "Input/Output Digit".into(),
        device_type: DeviceType::OwenDigitalIO,
        address: DeviceAddress::TcpIP(ip_address),
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
        values: Some(vec![
            Value {
                name: "Битовая маска состояния выходов".into(), // DO1 - DO8
                address: 468,
                direct: ValueDirect::Read(None),
                size: ValueSize::UINT8,
            },
            Value {
                name: "Битовая маска установки состояния выходов".into(),
                address: 470,
                direct: ValueDirect::Write,
                size: ValueSize::UINT8,
            },
        ]),
    }
}

pub fn make_invertor(ip_address: String) -> Device {
    Device {
        name: "Invertor".into(),
        address: DeviceAddress::TcpIP(ip_address), // "192.168.1.7"
        device_type: DeviceType::Invertor {
            functions: vec![
//                 InvertorFunc::DigitalOutput(0, 2), // Заданная частота достигнута
//                 InvertorFunc::DigitalOutput(0, 13), // Предупреждение о перегреве радиатора
//                 InvertorFunc::DigitalInput(0, 6), // Команда JOG (Разгон и Замедление)
//                 InvertorFunc::DigitalInput(0, 12), // Остановка на выбег/Пуск по рампе
//                 InvertorFunc::DigitalInput(0, 40), // Остановка на выбеге
                
            ]
        },
        sensors: Some(vec![
//             Group {
//                 name: "".into(),
//                 values: vec![],
//             },
        ]),
        values: {
            let add_simple_invertor_value = |name: &str, p: u16, adr: u16| Value {
                name: name.into(),
                address: p*256+adr,
                direct: ValueDirect::Write,
                size: ValueSize::UInt16Map(|v| v as f32/100_f32),
            };
            let add_simple_value_read = |p: u16, adr: u16, name: &str| Value {
                name: name.into(),
                address: p*256+adr,
                direct: ValueDirect::Read(None),
                size: ValueSize::UInt16Map(|v| v as f32/100_f32),
            };
            
            let mut reg = vec![
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
                
                add_simple_invertor_value("Стартовая частота",  1, 9), // 0.00 - 600.00
                add_simple_invertor_value("Верхнее ограничение выходной частота",  1, 10), // 0.00 - 600.00
                add_simple_invertor_value( "Нижнее ограничение выходной частота",  1, 11), // 0.00 - 600.00
                
                // 1.12 - 1.21 -- Временные параметры
                
                // 1.28 - 1.33 -- Частота пропуска (1, 2, 3)
                
                add_simple_invertor_value("Выбор режима разгона/замедления",  1, 44), // 0 - 4
                
            ];
            reg.append(&mut vec![
                // 2.1 - 2.8, 2.26 - 2.31 -- Дискретные входы // Значения 0 - 53
                    // add_simple_invertor_value("Выбор состояния для дискретных входов",  2, 12), // BitMap 16 bit
        //             Value {
        //                 name: "Выбор состояния для дискретных входов".into(),
        //                 address: 2*256+12,
        //                 direct: ValueDirect::Write,
        //                 size: ValueSize::BitMap, // 16 bit
        //             },
        //             add_simple_invertor_value("Скорость изменения частоты командами Up/Down",  2, 10), // 0.01 - 1.00
                    
                    // 2.36 - 2.46 -- Цифровые выходы // Значения 0 - 51
                    // f-2 -- Заданная частота достигнута
                    // f-13 -- Предупреждение о перегреве радиатора
                    // f-27-28 -- Выходной ток выше или ниже p-2-33
                    // f-35-39 -- Индикация ошибок p-6-23-26
                    
        //             Value {
        //                 name: "Выбор неактивного состояния для дискретных выходов".into(),
        //                 address: 2*256+18,
        //                 direct: ValueDirect::Write,
        //                 size: ValueSize::BitMap, // 16 bit
        //             },
            ]);
            
            // Part 5
            reg.append(&mut vec![
                add_simple_value_read(5, 31, "Наработка двигателя (мин)"),
                add_simple_value_read(5, 32, "Наработка двигателя (дни)"),
            ]);
            
            // Part 9 
            reg.append(&mut vec![
                add_simple_invertor_value("Заданная частота по коммуникационному интерфейсу", 9, 10), // 600.00
                
                add_simple_invertor_value("Индетификация коммуникационной платы", 9, 60),
                add_simple_invertor_value("IP конфигурация комм. платы", 9, 75),
                
                add_simple_invertor_value("IP адрес 1 комм. платы", 9, 76),
                add_simple_invertor_value("IP адрес 2 комм. платы", 9, 77),
                add_simple_invertor_value("IP адрес 3 комм. платы", 9, 78),
                add_simple_invertor_value("IP адрес 4 комм. платы", 9, 79),
            ]);
            
            let add_simple_value_bit = |num:u8, name: &str| ValueBit {name: name.into(), bit_num: num, bit_size: 1};
            // Part 20 Write
            reg.append(&mut vec![
                Value {
                    name: "2000H".into(),
                    address: 0x2000,
                    direct: ValueDirect::Write,
                    size: ValueSize::BitMap ( vec![
//                         ValueBit {
//                             name: "Run/Stop".into(),
//                             bit_num: 0,
//                             bit_size: 4, 
//                         },
                        add_simple_value_bit(1, "Stop"),
                        add_simple_value_bit(2, "Run"),
                        add_simple_value_bit(3, "Jog Run"),
                        ValueBit {
                            name: "Изменить направление вращения".into(),
                            bit_num: 4,
                            bit_size: 2,
                        },
                        ValueBit {
                            name: "Выбор времени разгона".into(),
                            bit_num: 8,
                            bit_size: 12-8, 
                        },
                        ValueBit {
                            name: "Разрешение функции bit6-11".into(),
                            bit_num: 12,
                            bit_size: 1, 
                        },
                        ValueBit {
                            name: "Изменение источника управления".into(),
                            bit_num: 13,
                            bit_size: 2, 
                        }, 
                    ]),
                },
                Value {
                    name: "Команда задания частоты".into(),
                    address: 0x2001,
                    direct: ValueDirect::Write,
                    size: ValueSize::UINT16,
                },
                Value {
                    name: "2002H".into(),
                    address: 0x2002,
                    direct: ValueDirect::Write,
                    size: ValueSize::BitMap ( vec![
                        add_simple_value_bit(0, "EF"),
                        add_simple_value_bit(1, "Сброс ошибки"),
                        add_simple_value_bit(2, "Внешняя пауза"),
                    ]),
                },
            ]);
            
            let add_simple_value_read = |adr: u16, name: &str| Value {
                name: name.into(), address: adr, 
                direct: ValueDirect::Read(None), size: ValueSize::UINT16,
            };
            // Part 21 ReadOnly
            reg.append(&mut vec![
                Value {
                    name: "Код ошибки".into(), // Pr.06-17 - 06.22
                    address: 0x2100,
                    direct: ValueDirect::Read(None), // interval
                    size: ValueSize::UINT16, // UINT32
                },
                Value {
                    name: "2119H".into(),
                    address: 0x2119,
                    direct: ValueDirect::Read(None),
                    size: ValueSize::BitMap (vec![
                        add_simple_value_bit(0, "Команда FWD"),
                        add_simple_value_bit(1, "Состояние привода"),
                        add_simple_value_bit(2, "Jog команда"),
                        add_simple_value_bit(3, "REV команда"),
                        add_simple_value_bit(4, "REV команда"),
                        add_simple_value_bit(8, "Задание частоты через интерфейс"),
                        add_simple_value_bit(9, "Задание частоты через аналоговый вход"),
                        add_simple_value_bit(10, "Управление приводом через интерфейс"),
                        add_simple_value_bit(12, "Копирование параметров из пульта разрешено"),
                    ]),
                },
                add_simple_value_read(0x2102, "Заданная частота (F)"),
                add_simple_value_read(0x2103, "Выходная частота (H)"),
                add_simple_value_read(0x2104, "Выходной ток (A)"),
                add_simple_value_read(0x2106, "Выходное напряжение (E)"),
                add_simple_value_read(0x2109, "Значение счётчика"),
                add_simple_value_read(0x211B, "Максимальная установленная частота"),
                add_simple_value_read(0x220F, "Температура радиатора"),
            ]);
            
            Some(reg)
        }
    }
}
