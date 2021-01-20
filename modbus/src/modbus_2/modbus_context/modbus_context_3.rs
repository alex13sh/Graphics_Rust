use modbus_rs::{tcp, Client};

use super::{Value, ModbusValues};
use super::init::DeviceAddress;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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
        if cfg!(not(feature = "test")) {
        if let DeviceAddress::TcpIP(txt) = address {
            let client = tcp::Transport::new(txt).ok()?;
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(Box::new(client))),
                ranges_address: get_ranges_value(&values, 8, true),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } else {
            None
        }
        } else {None}
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
//         use tokio::time::delay_for;
        use tokio::time::timeout;
        use std::time::Duration;
        
        let ctx = self.ctx.clone();
        let ranges = self.ranges_address.clone();
        for r in ranges {
            let buff = {
                let buff = async{ctx.lock().unwrap().read_holding_registers(*r.start(), *r.end() - *r.start()+1)}; // ?
//             let timeout = delay_for(Duration::from_millis(300));
                timeout(Duration::from_millis(300), buff).await??
            };
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        Ok(())
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
impl From<tokio::time::Elapsed> for DeviceError {
    fn from(_err: tokio::time::Elapsed) -> Self {
        DeviceError::TimeOut
    }
}
