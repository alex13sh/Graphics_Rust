use tokio_modbus::client::Context;
use futures::executor::block_on;

use super::{Value, ModbusValues};
use super::init::DeviceAddress;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::device::{
    DeviceError,
    UpdateReq, get_ranges_value, convert_modbusvalues_to_hashmap_address,
};

type Values = HashMap<u16, Arc<Value>>;
type RangeAddress = std::ops::RangeInclusive<u16>;

pub(crate) struct ModbusContext {
    ctx: Arc<Mutex<Context>>,
    is_rtu: bool,
    pub(crate) values: Values,
    ranges_address: Vec<RangeAddress>,
}

impl ModbusContext {
    
    pub fn new(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        use std::time::Duration;
        let num = if let DeviceAddress::TcpIp2Rtu(_, num) = address {*num} else {1};
        if cfg!(not(feature = "test")) {
        match address {
        DeviceAddress::TcpIP(txt) |
        DeviceAddress::TcpIp2Rtu(txt, _) => {
            use tokio_modbus::prelude::*;
            let socket_addr = (txt.to_owned()+":502").parse().ok()?;
            dbg!(&socket_addr);
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(block_on(tcp::connect_slave(socket_addr,  num.into())).ok()?)),
                is_rtu: num != 1,
                ranges_address: get_ranges_value(UpdateReq::ReadOnly.filter_values(&values), 8),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } _ => None,
        }
        } else {None}
    }
    
    pub async fn new_async(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        use std::time::Duration;
        let num = if let DeviceAddress::TcpIp2Rtu(_, num) = address {*num} else {1};
        if cfg!(not(feature = "test")) {
        match address {
        DeviceAddress::TcpIP(txt) |
        DeviceAddress::TcpIp2Rtu(txt, _) => {
            use tokio_modbus::prelude::*;
            let socket_addr = (txt.to_owned()+":502").parse().ok()?;
            dbg!(&socket_addr, num);
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(tcp::connect_slave(socket_addr, num.into()).await.ok()?)),
                is_rtu: num != 1,
                ranges_address: get_ranges_value(UpdateReq::ReadOnly.filter_values(&values), 8),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } _ => None,
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
    
    pub fn update(&self, ranges_address: Option<&Vec<RangeAddress>>) -> Result<(), DeviceError> {
        use tokio_modbus::client::Reader;
        let ranges_address = ranges_address.unwrap_or(&self.ranges_address);
        for r in ranges_address {
            let buff = block_on(async{self.ctx.lock().await.read_holding_registers(*r.start(), *r.end() - *r.start()+1).await})?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        Ok(())
    }
    
    pub async fn update_async(&self, ranges_address: Option<&Vec<RangeAddress>>) -> Result<(), DeviceError> {
        use tokio_modbus::client::Reader;
        use tokio::time::sleep;
        use tokio::time::timeout;
        use std::time::Duration;
        let ranges_address = ranges_address.unwrap_or(&self.ranges_address);
        for r in ranges_address {
            let buff = {
                let mut ctx = self.ctx.lock().await;
                let buff = ctx.read_holding_registers(*r.start(), *r.end() - *r.start()+1);
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
                let timeout = sleep(Duration::from_millis(
                    if self.is_rtu {300} else {100}
                ));
                let res = tokio::select! {
                buff = buff => Ok(buff),
                _ = timeout => Err(DeviceError::TimeOut),
                };
                res?
            };
            if let Ok(buff) = buff {
                Self::update_impl(&self.values, r.clone(), buff);
            } else {
                println!("-> Range ({:?})", r);
            }
        }
        Ok(())
    }
    
    pub(crate) fn is_busy(&self) -> bool {
//         self.ctx.is_poisoned()
        false
    }
    
    pub(crate) fn set_value(&self, v: &Value) -> Result<(), DeviceError> {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::Writer;
        match v.size.size() {
        1 => block_on(async{self.ctx.lock().await.write_single_register(v.address(), v.value() as u16).await})?,
        2 => {
            block_on(async{self.ctx.lock().await.write_single_register(v.address(), v.value() as u16).await})?;
            block_on(async{self.ctx.lock().await.write_single_register(v.address()+1, (v.value()>>16) as u16).await})?;
        },
        _ => {}
        };
        Ok(())
    }
    pub(crate) fn get_value(&self, v: &Value) -> Result<(), DeviceError>  {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::Reader;
        match v.size.size() {
        1 => v.update_value(block_on(async{self.ctx.lock().await.read_holding_registers(v.address(), 1).await})?[0] as u32),
        2 => {
            let buf = block_on(async{self.ctx.lock().await.read_holding_registers(v.address(), 2).await})?;
            v.update_value((buf[0] as u32) | (buf[1] as u32)<<16);
        },
        _ => {}
        };
        Ok(())
    }
}
