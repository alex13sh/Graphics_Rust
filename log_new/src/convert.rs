pub use value::value_date_convert;

pub mod value {
    use crate::value::*;
    pub fn value_date_convert<V, U>(from: ValueDate<V>) -> ValueDate<U> 
    where U: From<V>
    {
        ValueDate {
            date_time: from.date_time,
            value: from.value.into(),
        }
    }
    
    pub fn hash_to_names(hash: &str) -> (u16, String, String) {
        match hash {
        "Температура статора дв. М2/value" => (1, "МВ210-101".into(), "Температура статора дв. М2".into()),
        "Температура верх подшипника дв. М2/value" => (1, "МВ210-101".into(), "Температура верх подшипника дв. М2".into()),
        "Температура нижн подшипника дв. М2/value" => (1, "МВ210-101".into(), "Температура нижн подшипника дв. М2".into()),
        "Температура статора двигатель М1/value" => (1, "МВ210-101".into(), "Температура статора двигатель М1".into()),
        "Температура масла на верхн. выходе дв. М1/value" => (1, "МВ210-101".into(), "Температура масла на верхн. выходе дв. М1".into()),
        "Температура масла на нижн. выходе дв. М1/value" => (1, "МВ210-101".into(), "Температура масла на нижн. выходе дв. М1".into()),
        "Температура масла на выходе маслостанции/value" => (1, "МВ210-101".into(), "Температура масла на выходе маслостанции".into()),
        "Давление масла на выходе маслостанции/value" => (2, "МВ110-24.8АС".into(), "Давление масла на выходе маслостанции".into()),
        "Давление воздуха компрессора/value" => (2, "МВ110-24.8АС".into(), "Давление воздуха компрессора".into()),
        "Разрежение воздуха в системе/value" => (2, "МВ110-24.8АС".into(), "Разрежение воздуха в системе".into()),
        "Температура ротора Пирометр дв. М1/value" => (2, "МВ110-24.8АС".into(), "Температура ротора Пирометр дв. М1".into()),
        "Температура ротора Пирометр дв. М2/value" => (2, "МВ110-24.8АС".into(), "Температура ротора Пирометр дв. М2".into()),
        "Виброскорость дв. М1/value" => (2, "МВ110-24.8АС".into(), "Виброскорость дв. М1".into()),
        "Виброскорость дв. М2/value" => (2, "МВ110-24.8АС".into(), "Виброскорость дв. М2".into()),
        
        "Битовая маска состояния выходов" => (3, "МК210-302".into(), "Битовая маска состояния выходов".into()),
        "Битовая маска состояния входов" => (3, "МК210-302".into(), "Битовая маска состояния входов".into()),
        "Клапан ШК1 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК1 открыт".into()),
        "Клапан ШК1 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК1 закрыт".into()),
        "Клапан ШК2 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК2 открыт".into()),
        "Клапан ШК2 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК2 закрыт".into()),
        "Клапан ШК3 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК3 открыт".into()),
        "Клапан ШК3 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК3 закрыт".into()),
        "Клапан ШК4 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК4 открыт".into()),
        "Клапан ШК4 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК4 закрыт".into()),
        "Клапан ШК5 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК5 открыт".into()),
        "Клапан ШК5 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК5 закрыт".into()),
        "Клапан ШК6 открыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК6 открыт".into()),
        "Клапан ШК6 закрыт/read_bit_51" => (3, "МК210-302".into(), "Клапан ШК6 закрыт".into()),
        "Двигатель насоса вакуума 1/write_bit" => (3, "МК210-302".into(), "Двигатель насоса вакуума 1".into()),
        "Двигатель насоса вакуума 2/write_bit" => (3, "МК210-302".into(), "Двигатель насоса вакуума 2".into()),
        
        "Битовая маска состояния выходов" => (4, "МУ210-410".into(), "Битовая маска состояния выходов".into()),
        "Двигатель подачи материала в камеру/Частота высокочастотного ШИМ" => (4, "МУ210-410".into(), "Двигатель подачи материала в камеру".into()),
        "Направление вращения двигателя ШД/write_bit" => (4, "МУ210-410".into(), "Направление вращения двигателя ШД".into()),
        "Двигатель маслостанции М4/write_bit" => (4, "МУ210-410".into(), "Двигатель маслостанции М4".into()),
        "Двигатель компрессора воздуха/write_bit" => (4, "МУ210-410".into(), "Двигатель компрессора воздуха".into()),
        "Клапан нижнего контейнера/write_bit" => (4, "МУ210-410".into(), "Клапан нижнего контейнера".into()),
        "Клапан подачи материала/write_bit" => (4, "МУ210-410".into(), "Клапан подачи материала".into()),
        "Клапан помольной камеры/write_bit" => (4, "МУ210-410".into(), "Клапан помольной камеры".into()),
        "Клапан напуска/write_bit" => (4, "МУ210-410".into(), "Клапан напуска".into()),
        "Клапан верхнего контейнера/write_bit" => (4, "МУ210-410".into(), "Клапан верхнего контейнера".into()),
        "Клапан насоса М5/write_bit" => (4, "МУ210-410".into(), "Клапан насоса М5".into()),
        
        "ac4e9ff84c" => (5, "Invertor".into(), "Наработка двигателя (мин)".into()),
        "b735f11d88" => (5, "Invertor".into(), "Наработка двигателя (дни)".into()),
        "4c12e17ba3" => (5, "Invertor".into(), "Заданная частота (F)".into()),
        "4bd5c4e0a9" => (5, "Invertor".into(), "Скорость двигателя".into()),
        "5146ba6795" => (5, "Invertor".into(), "Выходной ток (A)".into()),
        "Напряжение на шине DC" => (5, "Invertor".into(), "Напряжение на шине DC".into()),
        "5369886757" => (5, "Invertor".into(), "Выходное напряжение (E)".into()),
        "2206H" => (5, "Invertor".into(), "Индикация текущей выходной мощности (P)".into()),
        "2207H" => (5, "Invertor".into(), "Индикация рассчитанной (с PG) скорости".into()),
        "5b28faeb8d" => (5, "Invertor".into(), "Температура радиатора".into()),
        
        "6) Invertor/ac4e9ff84c" => (6, "Invertor".into(), "Наработка двигателя (мин)".into()),
        "6) Invertor/b735f11d88" => (6, "Invertor".into(), "Наработка двигателя (дни)".into()),
        "6) Invertor/4c12e17ba3" => (6, "Invertor".into(), "Заданная частота (F)".into()),
        "6) Invertor/4bd5c4e0a9" => (6, "Invertor".into(), "Скорость двигателя".into()),
        "6) Invertor/5146ba6795" => (6, "Invertor".into(), "Выходной ток (A)".into()),
        "6) Invertor/Напряжение на шине DC" => (6, "Invertor".into(), "Напряжение на шине DC".into()),
        "6) Invertor/5369886757" => (6, "Invertor".into(), "Выходное напряжение (E)".into()),
        "6) Invertor/2206H" => (6, "Invertor".into(), "Индикация текущей выходной мощности (P)".into()),
        "6) Invertor/2207H" => (6, "Invertor".into(), "Индикация рассчитанной (с PG) скорости".into()),
        "6) Invertor/5b28faeb8d" => (6, "Invertor".into(), "Температура радиатора".into()),
        
        _ => (0, "".into(), "".into()),
        }
    }
}

pub mod stream {
    use crate::value::*;
    use futures::{Stream, StreamExt};
    pub fn raw_to_elk(raw_values: impl Stream<Item=LogValueRawOld> ) -> impl Stream<Item=LogValueHum> {
        raw_values.map(|v| super::value::value_date_convert(v))
    }
    
    pub fn values_to_line<V>(values: impl Stream<Item=V>) -> impl Stream<Item=ValueDate<V>> {
    
    }
}
