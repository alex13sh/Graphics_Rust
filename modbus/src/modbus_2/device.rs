// use super::Value;
use super::Sensor;
use super::{Value, ModbusValues};

use super::init::{DeviceType};
use super::init::Device as DeviceInit;
use super::init::ValueGroup as SensorInit;
use super::init::{ValueDirect, ValueSize};

use std::sync::Arc;

#[derive(Debug)]
pub struct Device {
    name: String,
    sensors: Vec<Sensor>,
    values: Vec<Arc<Value>>,
    device_type: DeviceType
}

impl From<DeviceInit> for Device {
    fn from(d: DeviceInit) -> Device {
        let typ = &d.device_type;
        let sens = d.sensors.unwrap_or(Vec::new()).into_iter().map(|s| typ.new_sensor(s));
        let values = d.values.unwrap_or(Vec::new()).into_iter().map(|v| Arc::new(Value::from(v)));
        Device {
            name: d.name,
            sensors: sens.collect(),
            device_type: d.device_type,
            values: values.collect(),
        }
    }
}

impl DeviceType {
    pub fn new_sensor(&self, s: SensorInit) -> Sensor { // TODO: Изменить тип сенсора
        let values;
        let value;
        match *self {
        Self::OwenAnalog => {
            match s {
            SensorInit::Sensor{pin, ..} => {
                values = create_values_owen_analog(pin);
                value = values.get("value").unwrap_or(&Arc::new(Value::default())).clone();
                
            },
            _ => {
                values = ModbusValues::new();
                value = Arc::new(Value::default());
            }
            }
        },
        Self::OwenDigitalIO => {
            match s {
            SensorInit::Sensor{pin, ..} => {
                values = create_values_owen_digital(pin, false);
                value = values.get("value").unwrap_or(&Arc::new(Value::default())).clone();
            },
            SensorInit::GroupPin{pin, ..} => {
                values = create_values_owen_digital(pin, true);
                value = Arc::new(Value::default());
            },
            _ => {
                values = ModbusValues::new();
                value = Arc::new(Value::default());
            }
            };
        }, 
        _ => {
            values = ModbusValues::new();
            value = Arc::new(Value::default());
        }
        };
        Sensor::new(s, values, value )
    }
    
}

fn create_values_owen_analog(pin: u8) -> ModbusValues {
    let mut v = Vec::new();
    let pin = pin as u16;
    v.push(Value::new("value_float", 4000+(pin-1)*3, ValueSize::FLOAT, ValueDirect::Read));
    v.push(Value::new("type", 4100+(pin-1)*16, ValueSize::UINT32, ValueDirect::Write)); // "Тип датчика"
    v.push(Value::new("point", 4103+(pin-1)*16, ValueSize::UINT16, ValueDirect::Write)); // "положение десятичной точки"
    v.push(Value::new("Верхняя граница", 4108+(pin-1)*16, ValueSize::FLOAT, ValueDirect::Write));
    v.push(Value::new("Нижняя граница", 4110+(pin-1)*16, ValueSize::FLOAT, ValueDirect::Write));
    v.push(Value::new("interval", 4113+(pin-1)*16, ValueSize::UINT16, ValueDirect::Write));
    
    v.into_iter().map(|v| Arc::new(v)).collect()
}
fn create_values_owen_digital(pin: u8, output: bool) -> ModbusValues {
    let mut v = Vec::new();
    let pin = pin as u16;
    if pin>=1 && pin<=8 && !output {v.push(Value::new("type_input", 64 +(pin-1), ValueSize::UINT16, ValueDirect::Write));} // "Дополнительный режим"
    if pin>=1 && pin<=12 {v.push(Value::new("filter", 96 +(pin-1), ValueSize::UINT16, ValueDirect::Write));} // "Фильтр"
    if pin>=1 && pin<=8 && !output {v.push(Value::new("interval", 128 +(pin-1), ValueSize::UINT16, ValueDirect::Write));} // "Дополнительный режим"
    if pin>=1 && pin<=12 {v.push(Value::new("value", 160 +(pin-1)*2, ValueSize::UINT32, ValueDirect::Read));} // "Значение входа в дополнительном режиме"
    if pin>=1 && pin<=8 && !output {v.push(Value::new("reset_value", 224 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write));} // "Сброс значения дополнительного режима"
    if pin>=9 && pin<=12 {v.push(Value::new("reset_counter", 232 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write));} // "Сброс значения счётчика импульсв"
    
    if pin>=1 && pin<=4 && output {
        v.push(Value::new("type_output", 272 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write)); // "Режим работы выхода"
        v.push(Value::new("Период ШИМ", 308 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write));
        v.push(Value::new("Коэффициент заполнения ШИМ", 340 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write));
        v.push(Value::new("Безопасное состояние выхода", 474 +(pin-1)*1, ValueSize::UINT16, ValueDirect::Write));
//         v.push(Value::new("Битовая маска состояния", 468, ValueSize::UINT8, ValueDirect::Read));
    }
    
    v.into_iter().map(|v| Arc::new(v)).collect()
}


impl std::iter::FromIterator<Arc<Value>> for ModbusValues {
    fn from_iter<I: IntoIterator<Item=Arc<Value>>>(iter: I) -> Self {
        let mut c = ModbusValues::new();

        for i in iter {
            c.insert(i.name().clone(), i);
        }

        c
    }
}
