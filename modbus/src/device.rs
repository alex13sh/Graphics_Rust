// #![allow(dead_code)]

use super::{Value, ModbusValues};

use super::init::{DeviceType, DeviceAddress, DeviceID};
use super::init::Device as DeviceInit;

use std::collections::HashMap;
use std::cell::{RefCell}; //, Cell, RefMut};
use std::sync::{ Arc};
use tokio::sync::Mutex;
use derivative::Derivative;

type ModbusContext = Arc<super::ModbusContext>;
// #[derive(Debug)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Device {
    name: DeviceID,
    address: DeviceAddress,
    #[derivative(Debug="ignore")]
    pub(super) values: ModbusValues,
    pub(super) device_type: DeviceType<Device>,
    #[derivative(Debug="ignore")]
    pub(super) ctx: Mutex< Option<ModbusContext> >,
}

use log::{info, trace, warn};

#[derive(Copy, Clone)]
pub enum UpdateReq {
    ReadOnly,
    // Логируемые значения (не только read only)
    Logable,
    // Все read only и все логируемые
    ReadOnlyOrLogable,
    Vibro,
    All,
}

impl UpdateReq {
    pub(crate) fn filter_values(&self, values: &ModbusValues) -> Vec<Arc<Value>> {
        match self {
        Self::ReadOnly => {
            values.iter()
                .filter(|v| v.1.is_read_only())
                .map(|(_, v)| v.clone())
                .collect()
        }
        Self::Logable => {
            values.iter()
                .filter(|v| v.1.is_log())
                .map(|(_, v)| v.clone())
                .collect()
        }
        Self::ReadOnlyOrLogable => {
            values.iter()
                .filter(|v| v.1.is_read_only() || v.1.is_log())
                .map(|(_, v)| v.clone())
                .collect()
        }
        Self::Vibro => {
            values.get_values_by_id(|id| id.sensor_name.starts_with("Виброскорость дв. "))
                .into_iter()
                .filter(|v| v.1.is_read_only())
                .map(|(_, v)| v)
                .collect()
        }
        Self::All => {
            values.iter()
                .map(|(_, v)| v.clone())
                .collect()
        }
        }
    }
    fn is_read_only(&self) -> bool {
        if let Self::ReadOnly = self {
            true
        } else {false}
    }
}

impl Device {
    // Для чего мне этот name нужен?? Для отображение полного имени, или только названия модуля?
    pub fn name(&self) -> String {
        self.name.to_string()
    }
    
    pub async fn connect(self: Arc<Self>) -> DeviceResult {
        trace!(target: "modbus::update::connect", "{:?}", self);
        if self.is_connect() {return Ok(());}
        
        let mut ctx = self.ctx.lock().await;
        let new_ctx = super::ModbusContext
            ::new_async(&self.address, &self.values).await.map(Arc::new); 
        
        if new_ctx.is_none() {
            return Err(DeviceError::ContextNull);
        }
        *ctx = new_ctx;
        Ok(())
    }
    pub fn is_connecting(&self) -> bool {
//         self.ctx.is_poisoned()
        false
    }

    async fn disconnect(&self) {
        *self.ctx.lock().await = None;
    }
    
    pub async fn update_async(self: Arc<Self>, req: UpdateReq) -> DeviceResult {
//         trace!("pub async fn update_async");

        let mut try_ctx = self.ctx.try_lock();
        let try_ctx = try_ctx.as_mut()
            .map_err(|_|DeviceError::ContextBusy)?;
        let ctx = try_ctx.as_ref()
            .ok_or(DeviceError::ContextNull)?;
        
//         info!("Device: {} - {:?}", self.name, self.address);
        let len = match (self.address.is_tcp_ip(), &req) {
            (true, UpdateReq::ReadOnly) => 8,
            (false, UpdateReq::ReadOnly) => 1,
            (false, UpdateReq::All) => 0,
            _ => 1,
        };
        let res = ctx.update_async(Some(&get_ranges_value(req.filter_values(&self.values), len))).await;
//         drop(ctx);
//         info!("-> res");
        if res.is_err() {
            log::error!(target: "modbus::update::update_async", "{:?}; {:?}", &res, self);
//             if self.address.is_tcp_ip() {
//                 self.disconnect().await;
                **try_ctx = None;
//             }
        }
        res
    }
    
    pub async fn update_new_values(self: Arc<Self>) -> DeviceResult {
        let ctx = self.context()?;
        for (_name, v) in self.values.iter() {
            if v.is_flag() {
                v.clear_flag();
                ctx.set_value(&v).await?;
            }
        }
        Ok(())
    }
    
    pub fn values(&self) -> Vec<Arc<Value>> {
        self.values.values().map(Arc::clone).collect()
    }
    pub fn values_map(&self) -> &ModbusValues {
        &self.values
    }
    pub(super) async fn context_async(&self) -> Result<ModbusContext, DeviceError> {
//         trace!("device get context");
        if let Some(ref ctx) = *self.ctx.lock().await {
            Ok(ctx.clone())
        } else {
            Err(DeviceError::ContextNull)
        }
    }
    pub(super) fn context(&self) -> Result<ModbusContext, DeviceError> {
//         trace!("device get context");
        if let Some(ref ctx) = *self.ctx.try_lock().map_err(|_|DeviceError::ContextBusy)? {
            Ok(ctx.clone())
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
    pub fn address(&self) -> &DeviceAddress {
        &self.address
    }
}

pub type DeviceResult = Result<(), DeviceError>;
#[derive(Debug, Clone)]
pub enum DeviceError {
    ContextNull,
    ContextBusy,
    TimeOut,
    ValueOut,
    ValueError,
    OtherError
}

use std::fmt;
impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<std::io::Error> for DeviceError {
    fn from(err: std::io::Error) -> Self {
        dbg!(err);
        DeviceError::ValueError
    }
}


impl From<DeviceInit> for Device {
    fn from(d: DeviceInit) -> Device {
        let typ: DeviceType<Device> = d.device_type.into();
        let mut values: ModbusValues  = d.values
            .into_iter().map(|v| Arc::new(Value::from(v)))
            .collect();
        
        {
            let new_values: Vec<_> = values.iter()
                .flat_map(|(_name, v)| v.get_values_bit())
                .map(Arc::new)
                .collect();
            let new_values = ModbusValues::from(new_values);
            values = values + new_values;
        }
        {
            let mut map: HashMap<_, Arc<Value>> = HashMap::new();
            for (id, v) in values.iter_mut() {
                if let Some(v_) = map.get(&v.address()) {
//                     dbg!(true);
                    if let Some(v) = Arc::get_mut(v) {
//                         dbg!(true);
                        (*v).merge_value(&v_);
                    }
                } else {map.insert(v.address(), v.clone());};
            }
        }
        
        trace!("Device from {}", d.name);
        
        Device {
            name: d.name,
            address: d.address.clone(),
            device_type: typ,
//             ctx: Mutex::new(super::ModbusContext::new(&d.address, &values).map(Arc::new)),
            ctx: Mutex::new(None),
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

pub(super) fn convert_modbusvalues_to_hashmap_address(values: &ModbusValues) -> HashMap<u16, Arc<Value>> {
    values.iter().map(|v| (v.1.address(), v.1.clone())).collect()
}

pub(super) fn get_ranges_value(mut values: Vec<Arc<Value>>, empty_space: u8) -> Vec<std::ops::RangeInclusive<u16>> {
    let empty_space = empty_space as u16;
    
    values.sort_by(|a, b| a.address().cmp(&b.address()));
    if values.len() == 0 {
        return Vec::new();
    }
    
    let mut itr = values.into_iter();
    let v = itr.next().unwrap();
    let adr = v.address();
    let end = adr + v.size() as u16-1;
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
