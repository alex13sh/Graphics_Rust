#![allow(dead_code)]

// use super::Value;
use super::Sensor;
use super::{Value, ModbusValues, ModbusSensors};

use super::init::{DeviceType, DeviceAddress};
use super::init::Device as DeviceInit;
use super::init::ValueGroup as SensorInit;
use super::init::{ValueDirect, ValueSize};

use std::collections::HashMap;
use std::sync::Arc;
use std::cell::{RefCell}; //, Cell, RefMut};
use derivative::Derivative;

// #[derive(Debug)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Device {
    name: String,
    address: DeviceAddress,
    sensors: ModbusSensors,
    pub(super) values: ModbusValues,
    pub(super) device_type: DeviceType<Device>,
    #[derivative(Debug="ignore")]
    pub(super) ctx: Option<RefCell<ModbusContext>>,
}

impl Device {
    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn get_ranges_value(&self, empty_space: u8, read_only: bool) -> Vec<std::ops::Range<u16>> {
        let empty_space = empty_space as u16;
        
        let mut adrs: Vec<_> = self.values.iter().filter(|v| v.1.is_read_only() || !read_only ).map(|v| v.1.address()).collect();
        if adrs.len() == 0 {
            return Vec::new();
        }
        adrs.sort();
        let adrs = adrs;
//         dbg!(&adrs);
        
        let mut itr = adrs.into_iter();
        let adr = itr.next().unwrap();
        let mut res = vec![std::ops::Range { start: adr, end: adr }];
        let mut last_range = res.last_mut().unwrap();
        
        for adr in itr {
//             let end = last_range.end;
            if last_range.end +empty_space < adr {
                let r = std::ops::Range { start: adr, end: adr };
                res.push(r);
            } else {
                last_range.end = adr;
            }
            last_range = res.last_mut().unwrap();
        }
        res
    }
    pub fn update(&self) -> Result<(), DeviceError> {
        self.context()?.borrow_mut().update()?;
        Ok(())
    }
    pub fn values(&self) -> Vec<Arc<Value>> {
        self.values.values().map(Arc::clone).collect()
    }
    pub fn values_map(&self) -> &ModbusValues {
        &self.values
    }
    pub(super) fn context(&self) -> Result<&RefCell<ModbusContext>, DeviceError> {
        if let Some(ctx) = &self.ctx {
            Ok(ctx)
        } else {
            Err(DeviceError::ContextNull)
        }
    }
    pub fn is_connect(&self) -> bool {
        self.context().is_ok()
    }
    pub fn get_ip_address(&self) -> String {
        match &self.address {
        DeviceAddress::TcpIP(ip_address) => ip_address.clone(),
        _ => "None".into(),
        }
    }
}

#[derive(Debug)]
pub enum DeviceError {
    ContextNull,
    ValueOut,
    ValueError,
    OtherError
}

use std::fmt;
impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         use DeviceError::*;
//         match self {
//         ContextNull => write!(f, "ContextNull"),
//         ValueOut => write!(f, "ValueOut"),
//         ValueError => write!(f, "ValueError"),
//         }
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<std::io::Error> for DeviceError {
    fn from(err: std::io::Error) -> Self {
        dbg!(err);
        DeviceError::ValueError
    }
}

pub(super) struct ModbusContext {
    ctx: super::ModbusContext,
    pub(super) values: HashMap<u16, Arc<Value>>,
    ranges_address: Vec<std::ops::RangeInclusive<u16>>,
}

impl ModbusContext {
    pub fn new(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        if cfg!(not(feature = "test")) {
        if let DeviceAddress::TcpIP(txt) = address {
            use tokio_modbus::prelude::*;
            let socket_addr = (txt.to_owned()+":502").parse().ok()?;
            dbg!(&socket_addr);
            
            Some(ModbusContext {
                ctx: sync::tcp::connect(socket_addr).ok()?,
                ranges_address: Self::get_ranges_value(&values, 8, true),
                values: values.iter().map(|v| (v.1.address(), v.1.clone())).collect(),
            })
        } else {
            None
        }
        } else {None}
    }
    pub fn update(&mut self) -> Result<(), DeviceError> {
        use tokio_modbus::client::sync::Reader;
        for r in &self.ranges_address {
            let buff = self.ctx.read_holding_registers(*r.start(), *r.end() - *r.start()+1)?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            let itr_buff = buff.into_iter();
            for (adr, v) in r.clone().zip(itr_buff) {
                if let Some(val) = self.values.get_mut(&adr) {
                    val.update_value(v as u32);
                }
                
            }
        }
        Ok(())
    }
    fn get_ranges_value(values: &ModbusValues, empty_space: u8, read_only: bool) -> Vec<std::ops::RangeInclusive<u16>> {
        let empty_space = empty_space as u16;
        if values.len() == 0 {
            return Vec::new();
        }
        
//         let mut adrs: Vec<_> = values.iter().filter(|v| v.1.is_read_only() || !read_only ).map(|v| v.1.address()).collect();
        let mut values: Vec<_> = values.iter().filter(|v| v.1.is_read_only() || !read_only ).map(|(_, v)| v.clone()).collect();
        values.sort_by(|a, b| a.address().cmp(&b.address()));
        let values = values;
        
        let mut itr = values.into_iter();
        let v = itr.next().unwrap();
        let adr = v.address();
        let end = adr + v.size() as u16;
        let mut res = vec![std::ops::Range { start: adr, end: end }];
        let mut last_range = res.last_mut().unwrap();
        
        for v in itr {
            let adr = v.address();
            let end = adr + v.size() as u16 -1;
            if last_range.end +empty_space < adr {
                let r = std::ops::Range { start: adr, end: end };
                res.push(r);
            } else {
                last_range.end = end;
            }
            last_range = res.last_mut().unwrap();
        }
        res.into_iter().map(|r| std::ops::RangeInclusive::new(r.start, r.end)).collect()
    }
    
    pub(super) fn set_value(&mut self, v: &Value) -> Result<(), DeviceError> {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::sync::Writer;
        match v.size.size() {
        1 => self.ctx.write_single_register(v.address(), v.value() as u16)?,
        2 => {
            self.ctx.write_single_register(v.address(), v.value() as u16)?;
            self.ctx.write_single_register(v.address()+1, (v.value()>>16) as u16)?;
        },
        _ => {}
        };
        Ok(())
    }
    pub(super) fn get_value(&mut self, v: &Value) -> Result<(), DeviceError>  {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::sync::Reader;
        match v.size.size() {
        1 => v.update_value(self.ctx.read_holding_registers(v.address(), 1)?[0] as u32),
        2 => {
            let buf = self.ctx.read_holding_registers(v.address(), 2)?;
            v.update_value((buf[0] as u32) | (buf[1] as u32)<<16);
        },
        _ => {}
        };
        Ok(())
    }
}

impl From<DeviceInit> for Device {
    fn from(d: DeviceInit) -> Device {
        let typ: DeviceType<Device> = d.device_type.into();
        let ref_typ = &typ;
        let sens = d.sensors.unwrap_or(Vec::new())
            .into_iter().map(|s| ref_typ.new_sensor(s));
        let values = d.values.unwrap_or(Vec::new())
            .into_iter().map(|v| Arc::new(Value::from(v)));
        
        let mut values: ModbusValues = values.collect();
        let sens: ModbusSensors = sens.collect();
        for s in sens.values() {
            for v in s.values().values() {
                values.insert(s.name().clone()+"/"+v.name(),v.clone());
            };
        };
        
        Device {
            name: d.name,
            address: d.address.clone(),
            sensors: sens,
            device_type: typ,
            ctx: ModbusContext::new(&d.address, &values).map(RefCell::new),
            values: values,
        }
    }
}

impl From<DeviceType<DeviceInit>> for DeviceType<Device> {
    fn from(dt: DeviceType<DeviceInit>) -> Self {
        match dt {
        DeviceType::<DeviceInit>::OwenAnalog => DeviceType::<Device>::OwenAnalog,
        DeviceType::<DeviceInit>::OwenDigitalIO => DeviceType::<Device>::OwenDigitalIO,
        DeviceType::<DeviceInit>::Invertor {functions} => DeviceType::<Device>::Invertor {functions:functions},
        DeviceType::<DeviceInit>::Convertor {devices} => {
            DeviceType::<Device>::Convertor {
                devices: devices.into_iter().map(|d| Device::from(d)).collect()
            }
        },
        }
    }
}

impl DeviceType<Device> {
    pub fn new_sensor(&self, s: SensorInit) -> Sensor {
        let values;
        let value;
        match *self {
        Self::OwenAnalog => {
            match s {
            SensorInit::Sensor{pin, value_error, ..} => {
                values = create_values_owen_analog(pin, value_error);
//                 value = values.get("value").unwrap_or(&None)).clone();
                value = None;
                
            },
            _ => {
                values = ModbusValues::new();
                value = None;
            }
            }
        },
        Self::OwenDigitalIO => {
            match s {
            SensorInit::Sensor{pin, ..} => {
                values = create_values_owen_digital(pin, false);
//                 value = values.get("value").unwrap_or(&Arc::new(Value::default())).clone();
                value = None;
            },
            SensorInit::GroupPin{pin, ..} => {
                values = create_values_owen_digital(pin, true);
                value = None;
            },
            _ => {
                values = ModbusValues::new();
                value = None;
            }
            };
        }, 
        _ => {
            values = ModbusValues::new();
            value = None;
        }
        };
        Sensor::new(s, values, value )
    }
}

fn create_values_owen_analog(pin: u8, err: crate::ValueError) -> ModbusValues {
    let mut v = Vec::new();
    let pin = pin as u16;
    v.push(Value::new("value_float", 4000+(pin-1)*3, ValueSize::FLOAT, ValueDirect::Read(Some(err))));
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
    if pin>=1 && pin<=12 {v.push(Value::new("value", 160 +(pin-1)*2, ValueSize::UINT32, ValueDirect::Read(None)));} // "Значение входа в дополнительном режиме"
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
impl std::iter::FromIterator<Sensor> for ModbusSensors {
    fn from_iter<I: IntoIterator<Item=Sensor>>(iter: I) -> Self {
        let mut c = ModbusSensors::new();

        for i in iter {
            c.insert(i.name().clone(), Arc::new(i));
        }

        c
    }
}
