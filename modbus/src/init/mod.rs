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
//     make_owen_analog("192.168.1.5".into()),
    make_io_digit("192.168.1.3".into()),
    make_invertor("192.168.1.5".into()),
    ]
}

fn make_value (name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Value {
    Value {
        name: name.into(),
        address: address,
        direct: direct,
        size: size,
        log: None,
    }
}
pub fn make_owen_analogs(ip_address: String) -> Vec<Device> {
    use SensorAnalogType::*;
    use ValueGroup::*;
    use sensor::SensorValues as SV;
    
    // (name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Self {
    
    let make_values = |pin: u16, err: ValueError, val_size: ValueSize| vec![
        Value {
            log: Log::hash(&format!("OwenAnalog/{}/value", pin)),
            .. make_value("value_float", 4000+(pin-1)*3, val_size, ValueDirect::Read(Some(err)))
        },
        make_value("type", 4100+(pin-1)*16, ValueSize::UINT32, ValueDirect::Write), // "Тип датчика"
        make_value("point", 4103+(pin-1)*16, ValueSize::UINT16, ValueDirect::Write), // "положение десятичной точки"
        make_value("Верхняя граница", 4108+(pin-1)*16, ValueSize::FLOAT, ValueDirect::Write),
        make_value("Нижняя граница", 4110+(pin-1)*16, ValueSize::FLOAT, ValueDirect::Write),
        make_value("interval", 4113+(pin-1)*16, ValueSize::UINT16, ValueDirect::Write),
    ];
    
    let make_sensor = |pin, name: &str, value_error: (i32, i32)| SV {
            name: name.into(),
            pin: pin,
            interval: 800,
            value_error: value_error.into(),
            sensor_type: SensorType::Analog(Pt_100),
            values: make_values(pin as u16, value_error.into(), ValueSize::FLOAT),
    };
    let make_sensor_2 = |pin, name: &str, value_error: (i32, i32)| SV {
            sensor_type: SensorType::Analog(Amper_4_20),
            .. make_sensor(pin, name, value_error)
        };
    let make_sensor_davl = |pin, name: &str, value_error: (f32, f32)| {
        SV {
            sensor_type: SensorType::Analog(Amper_4_20),
            value_error: value_error.into(),
            values: make_values(pin as u16, value_error.into(), 
                ValueSize::FloatMap(|v|10_f32.powf(v*10.0-5.5))
            ),
            .. make_sensor(pin, name, (0,0))
        }
    };
    
    vec![
    Device {
        name: "1) МВ210-101".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIP(ip_address),
        sensors: Some(vec![
            SensorValues(make_sensor(1, "Температура Статора дв.1", (60, 85))),
            SensorValues(make_sensor(2, "Температура масла на выходе 1 дв. Низ", (60, 85))), // <<-- ValueError
            SensorValues(make_sensor(3, "Температура масла на выходе 2 дв. Низ", (60, 85))), // <<-- ValueError
            SensorValues(make_sensor(4, "Температура масла на выходе маслостанции", (60, 85))), // <<-- ValueError
            SensorValues(make_sensor(5, "Температура Статора дв.2", (60, 85))),
            SensorValues(make_sensor(6, "Температура Пер.Под.", (60, 80))),
            SensorValues(make_sensor(7, "Температура Зад.Под.", (60, 80))),
            
        ]),
        values: None,
    },
    Device {
        name: "2) МВ110-24.8АС".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIP("192.168.1.11".into()), // <<--
        
        sensors: Some(vec![
            SensorValues(make_sensor_davl(1, "Давление масла верхний подшипник", (0.1, 0.5))),
            SensorValues(make_sensor_davl(2, "Давление масла нижний подшипник", (0.1, 0.5))),
            SensorValues(make_sensor_davl(3, "Давление воздуха компрессора", (0.1, 0.5))),
            SensorValues( SV {
                sensor_type: SensorType::Vibra(Amper_4_20),
                .. make_sensor(4, "Разрежение воздуха  в системе", (100, 106))
            }),
            
            SensorValues(make_sensor_2(5, "Температура Ротора дв.1", (60, 90))),
            SensorValues(make_sensor_2(6, "Температура Ротора дв.2", (60, 90))),
            
            SensorValues(make_sensor_2(7, "Вибродатчик дв.1", (10, 16))),
            SensorValues(make_sensor_2(8, "Вибродатчик дв.2", (10, 16))),
        ]),
        values: None,
    }
    ]
}

pub fn make_io_digit(ip_address: String) -> Device {
    use GroupValueType::DigitalOutput as DO;
    use ValueGroup::*;
    use sensor::SensorValues as SV;
    use sensor::GroupPinValues as GV;
    
    let make_values = |_pin: u16, _output: bool| vec![
    
    ];
    
    let _make_group = |pin: u8, name: &str, typ| GV {
        name: name.into(),
        group_type: typ,
        pin: pin, 
        values: make_values(pin as u16, false),
    };
    
    let make_counter = |pin: u16, name: &str, value_error: (i32, i32)| SV{ 
        name: name.into(),
        sensor_type: SensorType::Counter(0),
        pin: pin as u8, interval: 2,
        value_error: value_error.into(),
        values: vec![ // pin = 0; // pin - 1 = 0 - 1
            make_value("value", 160 +(pin-1)*2, ValueSize::UINT32, ValueDirect::Read(Some((333, 433).into()))),
            make_value("interval", 128 +(pin-1), ValueSize::UINT16, ValueDirect::Write),
            make_value("type_input", 64 +(pin-1), ValueSize::UINT16, ValueDirect::Write), // "Дополнительный режим"
            make_value("reset_counter", 232 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write), // "Сброс значения счётчика импульсв"
        ]
    };
    
    Device {
        name: "3) МК210-302".into(),
        device_type: DeviceType::OwenDigitalIO,
        address: DeviceAddress::TcpIP(ip_address),
        sensors: Some(vec![
            SensorValues(make_counter(1, "Скорость ротора дв. Верх", (333, 433))),
            
//             GroupPinValues( make_group(1, "Клапан 24В", DO(false)) ),
//             GroupPinValues( make_group(2, "Клапан 2", DO(false)) ),
//             GroupPinValues( make_group(3, "Насос", DO(false)) ),
        ]),
        values: Some(vec![
            Value {
                name: "Битовая маска состояния выходов".into(), // DO1 - DO8
                address: 468,
                direct: ValueDirect::Read(None),
                size: ValueSize::UINT8,
                log: Log::hash("308e553d36"),
            },
            make_value("Битовая маска установки состояния выходов", 470, ValueSize::UINT8, ValueDirect::Write),
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
                size: ValueSize::UInt16Map(|v| v as f32/10_f32),
                log: None,
            };
            let add_simple_value_read = |hash: &str, p: u16, adr: u16, name: &str| Value {
                name: name.into(),
                address: p*256+adr,
                direct: ValueDirect::Read(None),
                size: ValueSize::UInt16Map(|v| v as f32/10_f32),
                log: Log::hash(hash),
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
                add_simple_value_read("ac4e9ff84c", 5, 31, "Наработка двигателя (мин)"),
                add_simple_value_read("b735f11d88", 5, 32, "Наработка двигателя (дни)"),
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
                    log: None,
                },
                Value {
                    name: "Команда задания частоты".into(),
                    address: 0x2001,
                    direct: ValueDirect::Write,
                    size: ValueSize::UINT16,
                    log: None,
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
                    log: None,
                },
            ]);
            
            let add_simple_value_read_speed = |hash: &str, adr: u16, name: &str| Value {
                name: name.into(), address: adr, 
                direct: ValueDirect::Read(None), size: ValueSize::UInt16Map(|v| v as f32/100_f32*60_f32),
                log: Log::hash(hash),
            };
            let add_simple_value_read_100 = |hash: &str, adr: u16, name: &str| Value {
                name: name.into(), address: adr, 
                direct: ValueDirect::Read(None), size: ValueSize::UInt16Map(|v| v as f32/100_f32),
                log: Log::hash(hash),
            };
            let add_simple_value_read_10 = |hash: &str, adr: u16, name: &str| Value {
                name: name.into(), address: adr, 
                direct: ValueDirect::Read(None), size: ValueSize::UInt16Map(|v| v as f32/10_f32),
                log: Log::hash(hash),
            };
            // Part 21 ReadOnly
            reg.append(&mut vec![
                Value {
                    name: "Код ошибки".into(), // Pr.06-17 - 06.22
                    address: 0x2100,
                    direct: ValueDirect::Read(None), // interval
                    size: ValueSize::UINT16, // UINT32
                    log: None,
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
                    log: None,
                },
                add_simple_value_read_100("4c12e17ba3", 0x2102, "Заданная частота (F)"),
                add_simple_value_read_speed("4bd5c4e0a9", 0x2103, "Выходная частота (H)"),
                add_simple_value_read_100("5146ba6795", 0x2104, "Выходной ток (A)"),
                add_simple_value_read_100("5369886757", 0x2106, "Выходное напряжение (E)"),
//                 add_simple_value_read(0x2109, "Значение счётчика"),
//                 add_simple_value_read(0x211B, "Максимальная установленная частота"),
                add_simple_value_read_10("5b28faeb8d", 0x220F, "Температура радиатора"),
            ]);
            
            Some(reg)
        }
    }
}
