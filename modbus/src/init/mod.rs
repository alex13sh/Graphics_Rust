#![allow(dead_code)]

mod value;
mod device;
mod devices;

pub use value::*;
pub use device::*;


// #[test]
pub(crate) fn tst() {
    let d = init_devices();
    dbg!(d);
}

pub(crate) fn init_devices() -> Vec<Device> {    
    vec![
    make_owen_analog_1("192.168.1.11"),
    make_owen_analog_2("192.168.1.13"),
    make_i_digit("192.168.1.10".into()),
    make_o_digit("192.168.1.12".into()),
    make_invertor("192.168.1.5".into(), 5),
    make_invertor("192.168.1.6".into(), 6),
    ]
}

pub(super) fn make_value (name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Value {
    Value {
        name: name.into(),
        address: address,
        direct: direct,
        size: size,
        log: None,
    }
}

pub fn make_owen_analog_1(ip_addres: &str) -> Device {
    use devices::owen_analog::make_sensor;
    let make_values = |pin, name, err: (i32, i32)| make_sensor(pin, name, err.into(), ValueSize::FLOAT);
    
    Device {
        name: "1) МВ210-101".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIP(ip_addres.into()),
        values: Some(vec![
            make_values(1, "Температура статора двигатель М1", (60, 85)),
            make_values(2, "Температура масла на выходе дв. М1 Низ", (100, 120)), // <<-- ValueError
            make_values(3, "Температура масла на выходе дв. М2 Низ", (100, 120)), // <<-- ValueError
            make_values(4, "Температура масла на выходе маслостанции", (100, 120)), // <<-- ValueError
            make_values(5, "Температура статора двигатель М2", (60, 85)),
            make_values(6, "Температура подшипника дв. М1 верх", (60, 80)),
            make_values(7, "Температура подшипника дв. М2 верх", (60, 80)),
            
        ].into_iter().flatten().collect()),
    }
}

pub fn make_owen_analog_2(ip_addres: &str) -> Device {
    use devices::owen_analog::make_sensor_rtu as make_values;
    
    let make_sensor = |pin, name: &str, value_error: (i32, i32)|  make_values(pin, name, value_error.into(), ValueSize::UInt16Map(|v|v as f32 /10.0));

    let make_sensor_davl = |pin, name: &str, value_error: (f32, f32)|
        make_values(pin, name, value_error.into(), 
            ValueSize::UInt16Map(|v|10_f32.powf(v as f32 *10.0-5.5))
        );
    
    let make_sensor_vibra = |pin, name: &str, value_error: (f32, f32)|
        make_values(pin, name, value_error.into(), 
            ValueSize::UInt16Map(|v| {
                if v>500 {dbg!(v);}
                v as f32 / 100.0
            })
        );
    
    Device {
        name: "2) МВ110-24.8АС".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIp2Rtu(ip_addres.into(), 11), // <<--
        
        values: Some(vec![
            make_sensor_davl(1, "Давление масла верхний подшипник", (0.1, 0.5)),
            make_sensor_davl(2, "Давление масла нижний подшипник", (0.1, 0.5)),
            make_sensor_davl(3, "Давление воздуха компрессора", (0.1, 0.5)),
            make_sensor(4, "Разрежение воздуха в системе", (100, 106)),
            
            make_sensor(5, "Температура ротора Пирометр дв. М1", (60, 90)),
            make_sensor(6, "Температура ротора Пирометр дв. М2", (60, 90)),
            
            make_sensor(8, "Вибродатчик дв. М2", (10, 16)),
            make_sensor_vibra(7, "Вибродатчик дв. М1", (10.0, 16.0)),
        ].into_iter().flatten().collect()),
    }
}

pub fn make_i_digit(ip_address: String) -> Device {
    use devices::make_value;
    use devices::owen_digit::{make_counter, make_read_bit};
    
    let prefix = format!("{}", "3) МК210-302");
    Device {
        name: "3) МК210-302".into(),
        device_type: DeviceType::OwenDigitalIO,
        address: DeviceAddress::TcpIP(ip_address),
        values: Some(vec![
            vec![
                Value {
                    name: format!("{}/{}", prefix,"Битовая маска состояния выходов"), // DO1 - DO8
                    address: 468,
                    direct: ValueDirect::Read(None),
                    size: ValueSize::UINT8,
                    log: Log::hash("Битовая маска состояния выходов"),
                },
                make_value(&prefix, "Битовая маска установки состояния выходов", 470, ValueSize::UINT8, ValueDirect::Write),
            ],
//             make_counter(1, "Скорость ротора дв. Верх", (333, 433)),

            make_counter(1, "Наличие потока нижний подшипник", (0, 0)),
            make_counter(2, "Наличие потока верхний подшипник", (0, 0)),
            make_read_bit(11, "Уровень масла в маслостанции 1"),
            make_read_bit(12, "Уровень масла в маслостанции 2"),
        ].into_iter().flatten().collect()),    
    }
}

pub fn make_o_digit(ip_address: String) -> Device {
    use devices::make_value;
    use devices::owen_digit::*;
        
    let prefix = format!("{}", "4) МУ210-410");
    Device {
        name: "4) МУ210-410".into(),
        device_type: DeviceType::OwenDigitalIO,
        address: DeviceAddress::TcpIP(ip_address),
        values: Some(vec![
            vec![
                Value {
                    name: format!("{}/{}", prefix,"Битовая маска состояния выходов"), // DO1 - DO8
                    address: 468,
                    direct: ValueDirect::Read(None),
                    size: ValueSize::UINT8,
                    log: Log::hash("Битовая маска состояния выходов"),
                },
                make_value(&prefix, "Битовая маска установки состояния выходов", 470, ValueSize::UINT8, ValueDirect::Write),
            ],
            make_shim(1, "Двигатель подачи материала в камеру"),
            make_klapan(2, "Направление вращения двигателя ШД" ),
            
//             make_shim(6, "Двигатель насоса вакуума М5"),
//             make_shim(7, "Двигатель маслостанции М4"),
//             make_shim(8, "Двигатель компрессора воздуха М3"),
            
//             make_klapan(8, "Двигатель компрессора воздуха М3" ), // "Насос" ??
            make_klapan(9, "Клапан насоса М5 вакуум"), // "Клапан 24В"
            make_klapan(10, "Клапан насоса М6 вакуум"), // "Клапан 2"
            make_klapan(12, "Клапан напуска воздуха" ), // "Насос" 
            make_klapan(11, "Клапан камеры" ),
            make_klapan(13, "Клапан подачи материала в камеру" ),
            make_klapan(14, "Клапан выгрузки материала из камеры" ),
            make_klapan(15, "Клапан дозатора" ),
        ].into_iter().flatten().collect()),
    }
}

pub fn make_invertor(ip_address: String, num: u8) -> Device {
    Device {
        name: format!("{}) Invertor", num),
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
                        add_simple_value_bit(4, "FWD"),
                        add_simple_value_bit(5, "REV"),
                        
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
