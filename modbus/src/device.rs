// #![allow(dead_code)]

use super::{Value, ModbusValues};

pub use super::init::{DeviceConfig, DeviceAddress, DeviceID};
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
    pub config: DeviceConfig,
    #[derivative(Debug="ignore")]
    pub(super) values: ModbusValues,
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
    
    pub fn id(&self) -> &DeviceID {
        &self.name
    }
    
    pub async fn connect(self: Arc<Self>) -> DeviceResult {
        trace!(target: "modbus::update::connect", "{:?}", self);
        if self.is_connect() {return Ok(());}
        
        *self.ctx.lock().await = super::ModbusContext
            ::new_async(&self.address, &self.values).await.map(Arc::new); 
        
        if !self.is_connect() {
            return Err(DeviceError::ContextNull);
        }
        Ok(())
    }
    pub fn is_connecting(&self) -> bool {
//         self.ctx.is_poisoned()
        false
    }

    pub async fn disconnect(&self) {
        *self.ctx.lock().await = None;
    }
    
    pub async fn reconnect(self: Arc<Self>) -> DeviceResult {
        log::trace!(target: "modbus::update::connect", "reconnect {:?}", self.id());
        println!("reconnect: {:?}", self.id());
        use std::time::Duration;
        use tokio::time::sleep;
        let timeout = Duration::from_millis(500);
        self.disconnect().await;
        for _ in 0..6 {
            let f = self.clone().connect();
            let f_timeout = sleep(timeout);
            tokio::select! {
            res = f => {
                log::trace!(target: "modbus::update::connect", "reconnect ok; {:?}", self.id());
                return res;
            },
            _ = f_timeout => {},
            };
        }
        log::error!(target: "modbus::update::connect", "reconnect timeout {:?}", self.id());
        println!("reconnect timeout: {:?}", self.id());
        Err(DeviceError::TimeOut)
    }

    pub async fn update_async(self: Arc<Self>, req: UpdateReq) -> DeviceResult {
        log::trace!(target: "modbus::update", "update_async - {:?}", self.id());

        let mut try_ctx = self.ctx.try_lock();
        let try_ctx = try_ctx.as_mut()
            .map_err(|_|DeviceError::ContextBusy)?;
        let ctx = try_ctx.as_ref()
            .ok_or(DeviceError::ContextNull)?;
        
//         info!("Device: {} - {:?}", self.name, self.address);
        let len = match (self.address.is_tcp_ip(), &req) {
            (true, UpdateReq::ReadOnlyOrLogable) => 8,
            (false, UpdateReq::ReadOnlyOrLogable) => 2,
            (false, UpdateReq::All) => 0,
            _ => 1,
        };
        let res = ctx.update_async(Some(&get_ranges_value(req.filter_values(&self.values), len))).await;
//         drop(ctx);
//         info!("-> res");
        if res.is_err() {
            log::error!(target: "modbus::update::update_async", "{:?}; {:?}", &res, self.id());
//             if self.address.is_tcp_ip() {
//                 self.disconnect().await;
                **try_ctx = None;
//             }
        }
        log::trace!(target: "modbus::update", "res update_async - {:?}", self.id());
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
       !matches!(self.context(), Err(DeviceError::ContextNull))
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
            config: d.config,
//             ctx: Mutex::new(super::ModbusContext::new(&d.address, &values).map(Arc::new)),
            ctx: Mutex::new(None),
            values: values,
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
