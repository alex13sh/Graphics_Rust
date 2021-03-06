use modbus_rs::{tcp, Client};

use super::{Value, ModbusValues};
use super::init::DeviceAddress;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use log::{info};

use super::device::{
    DeviceError,
    get_ranges_value, convert_modbusvalues_to_hashmap_address,
};

type Values = HashMap<u16, Arc<Value>>;
type RangeAddress = std::ops::RangeInclusive<u16>;

pub(crate) struct ModbusContext {
    ctx: Arc<Mutex<Box<dyn Client+ 'static + Send>>>,
    pub(crate) values: Values,
    ranges_address: Vec<RangeAddress>,
}

impl ModbusContext {
    pub fn new(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        use std::time::Duration;
        let num = if let DeviceAddress::TcpIp2Rtu(_, num) = address {num} else {&1};
        if cfg!(not(feature = "test")) {
        match address {
        DeviceAddress::TcpIP(txt) |
        DeviceAddress::TcpIp2Rtu(txt, _) => {
            let client = tcp::Transport::new_with_cfg(
                txt, tcp::Config {
                    tcp_connect_timeout: Some(Duration::from_millis(100)),
                    tcp_read_timeout: Some(Duration::from_millis(100)),
                    tcp_write_timeout: Some(Duration::from_millis(100)),
                    modbus_uid: *num,
                    .. Default::default()
                }).ok()?;
            dbg!(num);
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(Box::new(client))),
                ranges_address: get_ranges_value(&values, 8, true),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } _ => None,
        }
        } else {None}
    }
    
    pub async fn new_async(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        if let DeviceAddress::TcpIP(txt) = address {
        
            use std::thread;
            use tokio::sync::mpsc;
            let (s, mut rx) = mpsc::unbounded_channel();
            
            let txt_adr = txt.clone();
            thread::spawn(move || {
                use std::time::Duration;
                let client = tcp::Transport::new_with_cfg( &txt_adr, tcp::Config {
                    tcp_connect_timeout: Some(Duration::from_millis(100)),
                        tcp_read_timeout: Some(Duration::from_millis(100)),
                        tcp_write_timeout: Some(Duration::from_millis(100)),
                        .. Default::default()
                    });
                if let Ok(client) = client {
                    s.send(client);
                }
            });
        
            let client = rx.recv().await.unwrap();
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(Box::new(client))),
                ranges_address: get_ranges_value(&values, 8, true),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } else {
            None
        }
    }
    
    fn update_impl(values: &Values, r: RangeAddress, buff: Vec<u16>) {
        let itr_buff = buff.into_iter();
        let mut itr_zip = r.zip(itr_buff);
        while let Some((adr, v)) = itr_zip.next() {
//             for (adr, v) in itr_zip {
            if let Some(val) = values.get(&adr) {
                val.update_value(v as u32);
                if val.size() == 1 {
                    val.update_value(v as u32);
                } else if val.size() == 2 {
                    if let Some((_, v2)) = itr_zip.next() {
                        let value: u32 = ((v2 as u32) << 16) | v as u32;
                        val.update_value(value);
                    }
                }
            }
        }
    }
    
    pub fn update(&self) -> Result<(), DeviceError> {
        for r in &self.ranges_address {
            let buff = self.ctx.lock()?.read_holding_registers(*r.start(), *r.end() - *r.start()+1)?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        Ok(())
    }
    
    #[cfg(feature = "time")]
    pub async fn update_async(&self) -> Result<(), DeviceError> {
        info!("context_3 pub async fn update_async");
        use tokio::time::sleep;
//         use tokio::time::timeout;
        use std::time::Duration;
        use tokio::sync::mpsc;
        use std::thread;
        
        if self.ctx.is_poisoned() {
            return Err(DeviceError::ContextBusy)
        }
        let ranges = self.ranges_address.clone();
        for r in ranges {
            let buff = {
                let (s, mut rx) = mpsc::unbounded_channel();
                let r = r.clone();
                let ctx = self.ctx.clone();
                thread::spawn(move || {
                    let buff = ctx.try_lock().unwrap().read_holding_registers(*r.start(), *r.end() - *r.start()+1);
                    s.send(buff).unwrap();
                });
                let buff = rx.recv();
                 // ?
                let timeout = sleep(Duration::from_millis(10));
//                 timeout(Duration::from_millis(100), buff).await??
                let res = tokio::select! {
                buff = buff => {
                    info!("-> select buff");
                    Ok(buff.unwrap())
                },
                _ = timeout => {
                    info!("-> select timeout");
                    Err(DeviceError::TimeOut)
                    },
                };
                res??
            };
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        info!("\t <- pub async fn update_async");
        Ok(())
    }
    
    pub(crate) fn is_busy(&self) -> bool {
        self.ctx.is_poisoned()
    }
    
    pub(crate) fn set_value(&self, v: &Value) -> Result<(), DeviceError> {
        let mut ctx = self.ctx.lock()?;
        match v.size.size() {
        1 => ctx.write_single_register(v.address(), v.value() as u16)?,
        2 => {
            ctx.write_single_register(v.address(), v.value() as u16)?;
            ctx.write_single_register(v.address()+1, (v.value()>>16) as u16)?;
        }, _ => {}
        };
        Ok(())
    }
    pub(crate) fn get_value(&self, v: &Value) -> Result<(), DeviceError>  {
        let mut ctx = self.ctx.lock()?;
        match v.size.size() {
        1 => v.update_value(ctx.read_holding_registers(v.address(), 1)?[0] as u32),
        2 => {
            let buf = ctx.read_holding_registers(v.address(), 2)?;
            v.update_value((buf[0] as u32) | (buf[1] as u32)<<16);
        }, _ => {}
        };
        Ok(())
    }
}

impl From<modbus_rs::Error> for DeviceError {
    fn from(_err: modbus_rs::Error) -> Self {
        DeviceError::ValueError
    }
}

#[cfg(feature = "time")]
impl From<tokio::time::error::Elapsed> for DeviceError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        DeviceError::TimeOut
    }
}
