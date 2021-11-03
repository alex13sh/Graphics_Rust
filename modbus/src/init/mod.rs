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
    make_owen_analog_2("192.168.1.13", 11),
    make_i_digit("192.168.1.10".into()),
    make_o_digit("192.168.1.12".into()),
    make_invertor("192.168.1.5".into(), 5),
    make_invertor("192.168.1.6".into(), 6),
    ]
}

pub(super) fn make_value (name: &str, address: u16, size: ValueSize, direct: ValueDirect) -> Value {
    Value {
        name: name.into(),
        suffix_name: None,
        address: address,
        direct: direct,
        size: size,
        log: None,
    }
}

pub fn make_owen_analog_1(ip_addres: &str) -> Device {
    use devices::owen_analog::make_sensor_fn;
    let make_values = |pin, name, err: (i32, i32)|
        make_sensor_fn(pin, name, |v|{
            v.size(ValueSize::FLOAT)
            .direct(ValueDirect::read().err_max(err.into()))
        });
    
    Device {
        name: "1) МВ210-101".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIP(ip_addres.into()),
        values: Some(vec![
            make_values(1, "Температура статора дв. М2", (60, 85)),
            make_values(2, "Температура верх подшипника дв. М2", (60, 80)),
            make_values(3, "Температура нижн подшипника дв. М2", (60, 80)),
            make_values(4, "Температура статора двигатель М1", (60, 85)),
            make_values(5, "Температура масла на верхн. выходе дв. М1", (100, 120)), // <<-- ValueError
            make_values(6, "Температура масла на нижн. выходе дв. М1", (100, 120)), // <<-- ValueError
            make_values(7, "Температура масла на выходе маслостанции", (100, 120)), // <<-- ValueError
            
        ].into_iter().flatten().collect()),
    }
}

pub fn make_owen_analog_2(ip_addres: &str, id: u8) -> Device {
    use devices::owen_analog::make_sensor_rtu_fn;
    
    let make_sensor = |pin, name: &str, value_error: (i32, i32)|
        make_sensor_rtu_fn(pin, name, |v| {
            v.size(ValueSize::UInt16Map(|v|v as f32 /100.0))
            .direct(ValueDirect::read().err_max(value_error.into()))
//             .with_suffix(suffix)
        });

    let make_sensor_err_min_max = |pin, name, suffix, err_min: (f32, f32), err_max: (f32, f32)|
        make_sensor_rtu_fn(pin, name, |v| {
            v.size(ValueSize::UInt16Map(|v| v as f32 / 100.0))
            .direct(ValueDirect::read()
                .err_min(err_min.into())
                .err_max(err_max.into()))
            .with_suffix(suffix)
        });

    let make_sensor_davl = |pin, name: &str, err_max: (f32, f32)|
        make_sensor_rtu_fn(pin, name, |v| {
                //ValueSize::UInt16Map(|v|10_f32.powf(v as f32 *10.0-5.5))
                //ValueSize::UInt16Map(|v| v as f32 / 1000.0)
            v.size(ValueSize::UInt16Map(|v|10_f32.powf(v as f32/1000.0 -5.52)))
            .direct(ValueDirect::read()
                .err_max(err_max.into()))
            .with_suffix("мБар")
        });
    
    let make_sensor_vibra = |pin, name: &str, value_error: (f32, f32)|
        make_sensor_rtu_fn(pin, name, |v| {
            v.size(ValueSize::UInt16Map(|v| v as f32 / 100.0))
            .direct(ValueDirect::read()
                .err_max(value_error.into()))
            .with_suffix("мм/с")
        });

    Device {
        name: "2) МВ110-24.8АС".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIp2Rtu(ip_addres.into(), id),
        
        values: Some(vec![
            make_sensor_err_min_max(1, "Давление масла на выходе маслостанции", "атм", (1.45, 1.37), (8.0, 10.0)), // <<-- ??
            make_sensor_err_min_max(3, "Давление воздуха компрессора", "атм", (6.0, 5.0), (8.0, 9.0)),
            make_sensor_davl(4, "Разрежение воздуха в системе", (40.0, 50.0)),
            
            make_sensor(5, "Температура ротора Пирометр дв. М1", (60, 90)),
            make_sensor(6, "Температура ротора Пирометр дв. М2", (60, 90)),
            
            make_sensor_vibra(7, "Виброскорость дв. М1", (5.0, 6.0)),
            make_sensor_vibra(8, "Виброскорость дв. М2", (10.0, 16.0)),

            vec![
                make_value("Адрес датчика", 0x50, ValueSize::UINT16, ValueDirect::Write),
                make_value("Скорость обмена", 0x30, ValueSize::UINT16, ValueDirect::Write),
                make_value("Запись изменений", 0x78, ValueSize::UINT16, ValueDirect::Write),
            ]
        ].into_iter().flatten().collect()),
    }
}

pub fn make_pdu_rs(ip_addres: &str, id: u8) -> Device {
    Device {
        name: "PDU-RS".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIp2Rtu(ip_addres.into(), id), // <<--

        values: Some(vec![
            make_value("value", 0x898, ValueSize::UINT16, ValueDirect::read().err_max((100, 120).into()))
                .with_log(Log::hash("Значение уровня масла")), // <<---
            make_value("hight limit", 0x1486, ValueSize::UINT16, ValueDirect::read().err_max((100, 120).into())) // <<---
                .with_log(Log::hash("Верхний предел уровня масла")),
            make_value("low limit", 0x1487, ValueSize::UINT16, ValueDirect::read().err_max((100, 120).into())) // <<---
                .with_log(Log::hash("Нижний предел уровня масла")),
            make_value("Адрес датчика", 0x15E2, ValueSize::UINT16, ValueDirect::Write),
            make_value("Скорость обмена", 0x15E3, ValueSize::UINT16, ValueDirect::Write),
            make_value("Применить новые сетевые параметры", 0x15EB, ValueSize::UINT16, ValueDirect::Write),
        ]),
    }
}

pub fn make_mkon(ip_addres: &str, id: u8) -> Device {
    Device {
        name: "МКОН".into(),
        device_type: DeviceType::OwenAnalog,
        address: DeviceAddress::TcpIp2Rtu(ip_addres.into(), id), // <<--

        values: Some(vec![
            make_value("Скорость обмена", 0x0209, ValueSize::UINT16, ValueDirect::Write),
            make_value("Кол. стоп-битов", 0x020B, ValueSize::UINT16, ValueDirect::Write),
//             make_value("IP Address", 0x001A, ValueSize::UINT16, ValueDirect::read()),
        ]),
    }
}

pub fn make_i_digit(ip_address: String) -> Device {
    use devices::make_value;
    use devices::owen_digit::*;
    
    let prefix = format!("{}", "3) МК210-302");
    Device {
        name: "3) МК210-302".into(),
        device_type: DeviceType::OwenDigitalIO,
        address: DeviceAddress::TcpIP(ip_address),
        values: Some(vec![
            vec![
                Value::new(468, &format!("{}/{}", prefix,"Битовая маска состояния выходов")) // DO1 - DO8
                    .direct(ValueDirect::read())
                    .size(ValueSize::UINT8)
                    .with_log(Log::hash("Битовая маска состояния выходов")),
                Value::new(51, &format!("{}/{}", prefix,"Битовая маска состояния входов")) // DO1 - DO8
                    .direct(ValueDirect::read())
                    .size(ValueSize::UINT8)
                    .with_log(Log::hash("Битовая маска состояния входов")),
                make_value(&prefix, "Битовая маска установки состояния выходов", 470, ValueSize::UINT8, ValueDirect::Write),
            ],
            (0..12).map(|i| {
                make_read_bit_51(i+1, &format!("Клапан ШК{} {}", i/2+1,
                    if i%2==0 {"открыт"} else {"закрыт"}))
            }).flatten().collect(),
            // Клапана

            make_klapan(1, "Двигатель насоса вакуума 1"),
            make_klapan(2, "Двигатель насоса вакуума 2"),

        ].into_iter().flatten().collect()),    
    }
}

#[test]
fn test_klapan_input() {
    use devices::make_value;
    use devices::owen_digit::{make_counter, make_read_bit};
    let values: Vec<_> = (0..12).map(|i| {
                make_read_bit(i+1, &format!("Клапан ШК{} {}", i/2+1,
                    if i%2==0 {"открыт"} else {"закрыт"}))
        }).flatten().collect();
    dbg!(&values);
//     assert!(false);
    let names: Vec<_> = values.iter()
        .skip(1).step_by(2)
        .map(|v| v.name.clone()).collect();
    assert_eq!(names[0], "Клапан ШК1 открыт/bit");
    assert_eq!(names[1], "Клапан ШК1 закрыт/bit");
    assert_eq!(names[10], "Клапан ШК6 открыт/bit");
    assert_eq!(names[11], "Клапан ШК6 закрыт/bit");
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
                Value::new(468, &format!("{}/{}", prefix,"Битовая маска состояния выходов")) // DO1 - DO8
                    .direct(ValueDirect::read())
                    .size(ValueSize::UINT8)
                    .with_log(Log::hash("Битовая маска состояния выходов")),
                make_value(&prefix, "Битовая маска установки состояния выходов", 470, ValueSize::UINT8, ValueDirect::Write),
            ],
            make_shim(1, "Двигатель подачи материала в камеру"),
            make_klapan(2, "Направление вращения двигателя ШД" ),

            make_klapan(7, "Двигатель маслостанции М4"),
            make_klapan(8, "Двигатель компрессора воздуха"),

            make_klapan(9, "Клапан нижнего контейнера"), // "Клапан 24В"
            make_klapan(10, "Клапан подачи материала"), // "Клапан 2"
            make_klapan(13, "Клапан помольной камеры" ), // "Насос"
            make_klapan(12, "Клапан напуска" ),
            make_klapan(11, "Клапан верхнего контейнера"), //"Клапан подачи материала в камеру" ),
//             make_klapan(14, "Клапан выгрузки материала из камеры" ),
            make_klapan(14, "Клапан насоса М5"),//"Клапан дозатора" ),
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
            let add_invertor_value = |name: &str, p: u16, adr: u16|
                Value::new(p*256+adr, name)
                .size(ValueSize::UINT16);
            let add_float_invertor_value = |name: &str, p: u16, adr: u16, dot|
                Value::new(p*256+adr, name)
                .size(ValueSize::UInt16Dot(dot));
            let add_simple_invertor_value = |name: &str, p: u16, adr: u16|
                add_float_invertor_value(name, p, adr, 1);
            let add_simple_value_read = |hash: &str, p: u16, adr: u16, name: &str|
                Value::new(p*256+adr, name)
                .direct(ValueDirect::read())
                .size(ValueSize::UInt16Map(|v| v as f32/10_f32))
                .with_log(Log::hash(hash));

            // P0
            let mut reg = vec![
                add_invertor_value("Идентификационный код преобразователя частоты",  0, 0),
                add_float_invertor_value("Номинальный ток преобразователя частоты",  0,1, 2),
                add_invertor_value("Сброс параметров",  0, 2), // 0 - 10
                
                add_invertor_value("Режим управления",  0, 10), // 0 - 2
                add_invertor_value("Метод управления скоростью",  0, 11), // 0 - 3
                add_invertor_value("Режим позиционирования",  0, 12), // 0 - 1
                add_invertor_value("Метод управления моментом",  0, 13), // 0 - 2

                add_invertor_value("Режим работы привода",   0, 16), // 0 - 1
                add_invertor_value("Несущая частота ШИМ",    0, 17), // Таблица преобразований

                add_invertor_value("Управление направлением вращения двигателя",  0, 23), // 0 - 2
                add_invertor_value("Сбособ остановки",   0, 22), // 0 - 1
                
                add_invertor_value("Источник задания частоты",           0, 20), // 0 - 8 // 8 - Плата
                add_invertor_value("Источник команд управления",         0, 21), // 0 - 5 // 5 - Плата
                add_invertor_value("Источник задания частоты (HAND)",    0, 30), // 0 - 8 // 8 - Плата
                add_invertor_value("Источник команд управления (HAND)",    0, 31), // 0 - 8 // 8 - Плата

                add_float_invertor_value("Время усреднения показаний (Ток)",    0,48, 3),
            ];

            // P1
            reg.append(&mut vec![
                add_float_invertor_value("Максимальная выходная частота",      1,0, 2), // 50.0 - 600.0
                add_float_invertor_value("Номинальная частота двигателя",      1,1, 2), // 0.0 - 600.0
                add_float_invertor_value("Номинальное напряжение двигателя",   1,2, 1), // 0 - 255.0

                add_float_invertor_value("Промежуточная частота 1 характеристики V/f для двигателя 1",   1,3, 2), // 0.00 - 600.00
                add_float_invertor_value("Промежуточное напряжение 1 хар-ки V/f для двигателя 1",   1,4, 1), // 0.0 - 240.0
                add_float_invertor_value("Промежуточная частота 2 характеристики V/f для двигателя 1",   1,5, 2), // 0.00 - 600.00
                add_float_invertor_value("Промежуточное напряжение 2 хар-ки V/f для двигателя 1",   1,6, 1), // 0.0 - 240.0
                add_float_invertor_value("Минимальная частота характеристики V/f для двигателя 1",   1,7, 2), // 0.00 - 600.00
                add_float_invertor_value("Минимальное напряжение характеристики V/f для двигателя 1",   1,8, 1), // 0.0 - 240.0


                add_float_invertor_value("Стартовая частота",  1,9, 2), // 0.00 - 600.00
                add_float_invertor_value("Верхнее ограничение выходной частота",  1,10, 2), // 0.00 - 600.00
                add_float_invertor_value( "Нижнее ограничение выходной частота",  1,11, 2), // 0.00 - 600.00

                // 1.12 - 1.21 -- Временные параметры
                add_float_invertor_value( "Время разгона 1",    1,12, 2),
                add_float_invertor_value( "Время замедления 1", 1,13, 2),
                add_float_invertor_value( "Время разгона 2",    1,14, 2),
                add_float_invertor_value( "Время замедления 2", 1,15, 2),
                add_float_invertor_value( "Время разгона 3",    1,16, 2),
                add_float_invertor_value( "Время замедления 3", 1,17, 2),
                add_float_invertor_value( "Время разгона 4",    1,18, 2),
                add_float_invertor_value( "Время замедления 4", 1,19, 2),

                add_float_invertor_value( "Порог переключения между 1/4 времени разгона/замедления",  1,23, 2),

                add_float_invertor_value( "Длительность начального участка S-кривой разгона",   1,24, 2),
                add_float_invertor_value( "Длительность конечного участка S-кривой разгона",    1,25, 2),
                add_float_invertor_value( "Длительность начального участка S-кривой замедления",1,26, 2),
                add_float_invertor_value( "Длительность конечного участка S-кривой замедления", 1,27, 2),

                // 1.28 - 1.33 -- Частота пропуска (1, 2, 3)
                add_float_invertor_value( "Пропуск частоты 1 (верхняя граница)",    1,28, 2),
                add_float_invertor_value( "Пропуск частоты 1 (нижняя граница)",     1,29, 2),
                add_float_invertor_value( "Пропуск частоты 2 (верхняя граница)",    1,30, 2),
                add_float_invertor_value( "Пропуск частоты 2 (нижняя граница)",     1,31, 2),
                add_float_invertor_value( "Пропуск частоты 3 (верхняя граница)",    1,32, 2),
                add_float_invertor_value( "Пропуск частоты 3 (нижняя граница)",     1,33, 2),

                add_invertor_value( "Выбор режима нулевой скорости",    1,33),
                add_invertor_value( "Выбор характеристики V/f",     1,43),
                add_invertor_value("Выбор режима разгона/замедления",  1, 44), // 0 - 4
                add_float_invertor_value( "Дискретность установки времени разгона/ замедления и S-кривой",     1,45, 2),
                add_float_invertor_value( "Время для быстрой остановки с CAN open",     1,46, 2),
            ]);

            reg.append(&mut vec![
                add_invertor_value( "Режим оперативного управления", 2,0),
                add_invertor_value( "Режим изменения частоты командами UP/DOWN", 2,9),
                add_float_invertor_value( "Скорость изменения частоты командами UP/DOWN",     2,10, 2),
            ]);
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
                add_invertor_value( "Автотестирование двигателя",  5, 0),
                add_float_invertor_value( "Номинальный ток асинхронного двигателя 1(A)",  5,1, 2),
                add_float_invertor_value( "Номинальная мощность асинхронного двигателя 1 (кВт)",  5,2, 2),
                add_invertor_value( "Номинальная скорость асинхронного двигателя 1 (об/мин)",  5, 3),
                add_invertor_value( "Число полюсов асинхронного двигателя 1",  5, 4),
                add_float_invertor_value( "Ток холостого хода асинхронного двигателя 1 (A)",  5,5, 2),

                add_float_invertor_value( "Сопротивление статора (Rs) асинхронного двигателя 1",  5,6, 3),
                add_float_invertor_value( "Сопротивление ротора (Rr) асинхронного двигателя 1",  5,7, 3),

                add_float_invertor_value( "Взаимоиндуктивность (Lm) асинхронного двигателя 1",  5,8, 1),
                add_float_invertor_value( "Индуктивность статора (Lx) асинхронного двигателя 1",  5,9, 1),
//                 add_float_invertor_value( "Номинальный ток асинхронного двигателя 2(A)",  5,13, 2),

                add_float_invertor_value( "Частота, на которой происходит переключение «звезда»/ «треугольник»",  5,23, 2),
                add_invertor_value( "Переключение «звезда»/ «треугольник»",  5, 24),
                add_float_invertor_value( "Задержка при переключении «звезда»/ «треугольник»",  5,25, 3),

                add_simple_value_read("ac4e9ff84c", 5, 31, "Наработка двигателя (мин)"),
                add_simple_value_read("b735f11d88", 5, 32, "Наработка двигателя (дни)"),

                add_invertor_value( "Выбор между асинхронным двигателем и двигателем с постоянными магнитами",  5, 33),
                add_float_invertor_value( "Ном. ток двигателя с постоянными магнитами",  5,34, 2),
                add_float_invertor_value( "Ном. мощность двигателя с постоянными магнитами",  5,35, 2),
                add_invertor_value( "Ном. скорость двигателя с постоянными магнитами",  5, 36),
                add_invertor_value( "Количество полюсов двигателя с постоянными магнитами",  5, 37),
                add_simple_invertor_value( "Инерция двигателя с постоянными магнитами",  5, 38),
                add_float_invertor_value( "Сопротивление статора двигателя с постоянными магнитами",  5,39, 3),
                add_float_invertor_value( "Ld двигателя с постоянными магнитами",  5,40, 2),
                add_float_invertor_value( "Lq двигателя с постоянными магнитами",  5,41, 2),
                add_float_invertor_value( "Угол между магнитным полюсом и нулевой меткой датчика ОС",  5,42, 1),
                add_invertor_value( "Параметр Ke двигателя с постоянными магнитами",  5, 43),


            ]);

            // Проверка после резольвера:
            // 00.11
            // 06.03, 06.04
            // Part 6
            reg.append(&mut vec![

                add_simple_invertor_value( "Уровень ограничения перенапряжения",  6, 01),

                add_invertor_value( "Токоограничение при разгоне",  6, 03),
                add_invertor_value( "Токоограничение в установившемся режиме",  6, 04),
                add_invertor_value( "Выбор времени разгона/торможения при токоограничении в установившемся режиме",  6, 05),

                add_invertor_value( "Уровень ограничения тока",  6, 12),

                add_simple_invertor_value( "Уровень перегрева радиатора (OH)",  6, 15),
                add_invertor_value( "Порог ограничения для функций токоограничения",  6, 16),

                add_float_invertor_value( "Заданная частота при аварии",  6,31, 2),
                add_float_invertor_value( "Выходная частота при аварии",  6,32, 2),
                add_simple_invertor_value( "Выходное напряжение при аварии",  6, 33),
                add_simple_invertor_value( "Напряжение на шине DC при аварии",  6, 34),
                add_simple_invertor_value( "Выходной ток при аварии",  6, 35),

                add_invertor_value( "Снижение несущей частоты ШИМ",  6, 55),
            ]);
            
            // Part 7
            reg.append(&mut vec![
                add_float_invertor_value( "Уровень напряжения для включения тормозного транзистора",  7,0, 1),
                add_invertor_value( "Уровень тока при торможении постоянным током (DC Brake)",  7,1),
                add_float_invertor_value( "Время торможения постоянным током при старте",  7,2, 1),
                add_float_invertor_value( "Время торможения постоянным током при остановке",  7,3, 2),
                add_float_invertor_value( "Частота начала торможения постоянным током",  7,4, 2),
                add_invertor_value( "Реакция на кратковременное пропадания напряжения питания",  7,6),
                add_float_invertor_value( "Максимальное время пропадания напряжения",  7,7, 1),
                add_float_invertor_value( "Задержка поиска скорости после паузы",  7,8, 1),

                add_invertor_value( "Ограничение тока при поиске скорости",  7,9),
                add_invertor_value( "Поиск скорости при перезапуске после аварии",  7,10),
                add_invertor_value( "Количество автоперезапусков после аварии",  7,11),

                add_invertor_value( "Поиск скорости при пуске",  7,12),

                add_invertor_value( "Время замедления при пропадании напряжения питания (DEB)",  7,13),
                add_float_invertor_value( "Время возврата при DEB",  7,14, 1),

                add_float_invertor_value( "Задержка при разгоне",  7,15, 2),
                add_float_invertor_value( "Частота задержки при разгоне",  7,16, 2),
                add_float_invertor_value( "Задержка при замедлении",  7,17, 2),
                add_float_invertor_value( "Частота задержки при замедлении",  7,18, 2),

                add_invertor_value( "Управление встроенным вентилятор охлаждения",  7,19),
                add_invertor_value( "Внешний аварийный стоп (EF) и принудительный останов",  7,20),

                add_invertor_value( "Функция автоматического энергосбережения",  7,21),
                add_invertor_value( "Коэффициент автоматического энергосбережения",  7,22),
                add_invertor_value( "Функция автоматической регулировки выходного напряжения",  7, 23),

                add_float_invertor_value( "Постоянная времени компенсации момента (для V/f и SVC режима)",  7,24, 3),
                add_float_invertor_value( "Постоянная времени компенсации скольжения (для V/f и SVC режима)",  7,25, 3),
                add_invertor_value( "Уровень компенсации момента (для V/f и SVC режима)",  7, 26),
                add_float_invertor_value( "Уровень компенсации скольжения (для V/f и SVC режима)",  7,27, 2),

                add_float_invertor_value( "Уровень отклонения скольжения",  7,29, 1),
                add_float_invertor_value( "Время измерения отклонения скольжения",  7,30, 1),
                add_invertor_value( "Реакция на превышение скольжения",  7, 31),
                add_invertor_value( "Коэффициент компенсации неустойчивости вращения",  7, 32),

                add_invertor_value( "Коэффициент компенсации неустойчивости вращения",  7, 32),
            ]);
            // Part 8
            reg.append(&mut vec![
                add_invertor_value( "Вход для сигнала обратной связи ПИД",  8, 0),
                add_float_invertor_value( "Пропорциональный коэффициент (P)",  8,1, 1),
                add_float_invertor_value( "Интегральный коэффициент (I)",  8,2, 2),
                add_float_invertor_value( "Дифференциальный коэффициент (D)",  8,3, 2),
                add_float_invertor_value( "Верхнее ограничение интегрирования",  8,4, 1),
                add_float_invertor_value( "Ограничение выходной частоты при ПИД",  8,5, 1),
                add_float_invertor_value( "Задержка для ПИД",  8,7, 1),
                add_float_invertor_value( "Время обнаружения сигнала обратной связи",  8,8, 1),

                add_invertor_value( "Реакция на ошибку обратной связи",  8, 9),

                add_float_invertor_value( "Частота перехода в спящий режим",  8,10, 2),
                add_float_invertor_value( "Частота выхода из спящего режима",  8,11, 2),
                add_float_invertor_value( "Задержка входа в спящий режим",  8,12, 1),
            ]);
            // Part 9 
            reg.append(&mut vec![
                add_simple_invertor_value("Заданная частота по коммуникационному интерфейсу", 9, 10), // 600.00
                
//                 add_simple_invertor_value("Индетификация коммуникационной платы", 9, 60),
//                 add_simple_invertor_value("IP конфигурация комм. платы", 9, 75),
                
//                 add_simple_invertor_value("IP адрес 1 комм. платы", 9, 76),
//                 add_simple_invertor_value("IP адрес 2 комм. платы", 9, 77),
//                 add_simple_invertor_value("IP адрес 3 комм. платы", 9, 78),
//                 add_simple_invertor_value("IP адрес 4 комм. платы", 9, 79),
            ]);
            // Part 10
            reg.append(&mut vec![
            // 10.00-01
                add_invertor_value( "Выбор типа датчика обратной связи по скорости",  10, 00),
                add_invertor_value( "Число импульсов на оборот",  10, 01),
                add_invertor_value( "Выбор типа энкодера",  10, 02),
                // 10.02
            // 10.25
                add_simple_invertor_value( "Частота контроля скорости в режиме FOC",  10, 25),
                add_simple_invertor_value( "Минимальная частота на статоре при FOC",  10, 26),
                add_invertor_value( "Постоянная времени НЧ-фильтра FOC",  10, 27),

                add_invertor_value( "Коэффициент усиления времени нарастания тока возбуждения",  10, 28),

                add_float_invertor_value( "Коэффициент усиление Интегральный",      10,35, 2),
                add_float_invertor_value( "Коэффициент усиление Пропорциональный",  10,36, 2),

            ]);
            // Part 11
            reg.append(&mut vec![
                add_invertor_value( "Система управления",  11, 00),
                add_invertor_value( "Единицы инерции",  11, 01),
                add_float_invertor_value( "Частота переключения ASR1/ASR2",  11,02, 2),
                add_invertor_value( "ASR1 Полоса пропускания на низкой скорости",  11, 03),
                add_invertor_value( "ASR2 Полоса пропускания на высокой скорости",  11, 04),
                add_invertor_value( "Полоса пропускания на нулевой скорости",  11, 05),
                add_invertor_value( "ASR (Auto Speed Regulation) коэффициент (P) 1",  11, 06),
                add_float_invertor_value( "ASR (Auto Speed Regulation) коэффициент (I) 1",  11,07, 3),
                add_invertor_value( "ASR (Auto Speed Regulation) коэффициент (PI) 2",  11, 08),
                add_float_invertor_value( "ASR (Auto Speed Regulation) коэффициент (I) 2",  11,09, 3),
                add_invertor_value( "ASR (Auto Speed Regulation) коэффициент (P) для нулевой скорости",  11, 10),
                add_float_invertor_value( "ASR (Auto Speed Regulation) коэффициент (I) для нулевой скорости",  11,11, 3),
                add_invertor_value( "Усиление для ASR скорости прямой подачи",  11, 12),

                // 11.12-14 -- Обратная связь
                add_invertor_value( "PDFF усиление",  11, 13),
                add_float_invertor_value( "НЧ-фильтр для ASR выхода",  11,14, 3),
                add_invertor_value( "Глубина узкополосного режекторного фильтра",  11, 15),
                add_float_invertor_value( "Частота узкополосого режекторного фильтра",  11,16, 2),

                add_invertor_value( "Ограничение момента прямого вращения",  11, 17),
                add_invertor_value( "Ограничение тормозного момента прямого вращения",  11, 18),
                add_invertor_value( "Ограничение момента обратного вращения",  11, 19),
                add_invertor_value( "Ограничение тормозного момента обратного вращения",  11, 20),

                add_invertor_value( "Коэффициент ослабления поля двигателя 1",  11, 21),
//                 add_invertor_value( "Коэффициент ослабления поля двигателя 2",  11, 22),

                add_invertor_value( "Отклик скорости для области ослабления поля",  11, 23),
                add_float_invertor_value( "Коэффициент APR",  11,24, 2),
                add_invertor_value( "Коэффициент усиления APR прямой подачи",  11, 25),
                add_float_invertor_value( "Временная характеристика APR",  11,26, 2),
                add_invertor_value( "Макс. задание момента",  11, 27),
                add_invertor_value( "Источник смещения момента",  11, 28),

                add_float_invertor_value( "Смещение момента",  11,29, 1),
                add_float_invertor_value( "Высокое смещение момента",  11,30, 1),
                add_float_invertor_value( "Среднее смещение момента",  11,31, 1),
                add_float_invertor_value( "Малое смещение момента",  11,32, 1),

                add_invertor_value( "Источник задания момента",  11, 33),
                add_float_invertor_value( "Заданный момент",  11,34, 1),
                add_float_invertor_value( "НЧ-фильтр задания момента",  11,35, 3),

                add_invertor_value( "Выбор метода ограничения скорости",  11, 36),
                add_invertor_value( "Ограничение скорости прямого вращения (режим момента)",  11, 37),
                add_invertor_value( "Ограничение скорости обратного вращения (режим момента)",  11, 38),

                add_invertor_value( "Источник команд позиционирования в режиме 'точка к точке'",  11, 40),
                add_float_invertor_value( "Макс. частота в режиме позиционирования 'точка к точке'",  11,43, 2),
                add_float_invertor_value( "Время разгона при позиционировании 'точка к точке'",  11,44, 2),
                add_float_invertor_value( "Время замедления при позиционировании 'точка к точке'",  11,45, 2),
            ]);

            let add_simple_value_bit = |num:u8, name: &str| ValueBit {name: name.into(), bit_num: num, bit_size: 1};
            // Part 20 Write
            reg.append(&mut vec![
                Value::new(0x2000, "2000H")
                    .size(ValueSize::BitMap ( vec![
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
                    ])),
                Value::new(0x2001, "Команда задания частоты")
                    .size(ValueSize::UINT16),
                Value::new(0x2002, "2002H")
                    .size(ValueSize::BitMap ( vec![
                        add_simple_value_bit(0, "EF"),
                        add_simple_value_bit(1, "Сброс ошибки"),
                        add_simple_value_bit(2, "Внешняя пауза"),
                    ])),
            ]);
            
            let add_simple_value_read = |hash: &str, adr: u16, name: &str|
                Value::new(adr, name)
                .direct(ValueDirect::read())
                .with_log(Log::hash(hash));
            let add_simple_value_read_speed = |hash: &str, adr: u16, name: &str|
                add_simple_value_read(hash, adr, name)
                    .size(ValueSize::UInt16Map(|v| v as f32/100_f32*60_f32));
            let add_simple_value_read_100 = |hash: &str, adr: u16, name: &str|
                add_simple_value_read(hash, adr, name)
                    .size(ValueSize::UInt16Map(|v| v as f32/100_f32));
            let add_simple_value_read_10 = |hash: &str, adr: u16, name: &str|
                add_simple_value_read(hash, adr, name)
                    .size(ValueSize::UInt16Map(|v| v as f32/10_f32));
            // Part 21 ReadOnly
            reg.append(&mut vec![
                Value::new(0x2100, "Код ошибки") // Pr.06-17 - 06.22
                    .direct(ValueDirect::read()) // interval
                    .size(ValueSize::UINT16), // UINT32
                Value::new(0x2119, "2119H")
                    .direct(ValueDirect::read())
                    .size(ValueSize::BitMap (vec![
                        add_simple_value_bit(0, "Команда FWD"),
                        add_simple_value_bit(1, "Состояние привода"),
                        add_simple_value_bit(2, "Jog команда"),
                        add_simple_value_bit(3, "REV команда"),
                        add_simple_value_bit(4, "REV команда"),
                        add_simple_value_bit(8, "Задание частоты через интерфейс"),
                        add_simple_value_bit(9, "Задание частоты через аналоговый вход"),
                        add_simple_value_bit(10, "Управление приводом через интерфейс"),
                        add_simple_value_bit(12, "Копирование параметров из пульта разрешено"),
                    ])),
                add_simple_value_read_100("4c12e17ba3", 0x2102, "Заданная частота (F)").with_suffix("Герц"),
                add_simple_value_read_speed("4bd5c4e0a9", 0x2103, "Скорость двигателя").with_suffix("об./мин"), // fix me
                add_simple_value_read_100("5146ba6795", 0x2104, "Выходной ток (A)").with_suffix("А"),
                add_simple_value_read_100("Напряжение на шине DC", 0x2105, "Напряжение на шине DC"),
                add_simple_value_read_10("5369886757", 0x2106, "Выходное напряжение (E)"),
                add_simple_value_read_10("2206H", 0x2206, "Индикация текущей выходной мощности (P)").with_suffix("кВт"),
                add_simple_value_read("2207H", 0x2207, "Индикация рассчитанной (с PG) скорости").with_suffix("об./мин"),
//                 add_simple_value_read(0x2109, "Значение счётчика"),
//                 add_simple_value_read(0x211B, "Максимальная установленная частота"),
                add_simple_value_read_10("5b28faeb8d", 0x220F, "Температура радиатора"),
            ]);
            
            Some(reg)
        }
    }
}
