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
    
    pub async fn new_async(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        use std::time::Duration;
        let num = if let DeviceAddress::TcpIp2Rtu(_, num) = address {*num} else {1};
        if cfg!(not(feature = "test")) {
        match address {
        DeviceAddress::TcpIP(txt) |
        DeviceAddress::TcpIp2Rtu(txt, _) => {
            use tokio_modbus::prelude::*;
            let socket_addr = (txt.to_owned()+":502").parse().ok()?;
            log::trace!(target: "modbus::update::connect", "ModbusContext::new: adr: {:?}, num: {:?}", &socket_addr, num);
            let ctx = Arc::new(Mutex::new(tcp::connect_slave(socket_addr, num.into()).await.ok()?));
            log::trace!(target: "modbus::update::connect", "ctx ready  adr: {:?}, num: {:?}", &socket_addr, num);
            Some(ModbusContext {
                ctx: ctx,
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
    
    pub async fn update_async(&self, ranges_address: Option<&Vec<RangeAddress>>) -> Result<(), DeviceError> {
        use tokio_modbus::client::Reader;
        use tokio::time::sleep;
        use tokio::time::timeout;
        use std::time::Duration;
        let ranges_address = ranges_address.unwrap_or(&self.ranges_address);
        
        let timeout = Duration::from_millis(
            if self.is_rtu {300*ranges_address.len()} else {100} as u64
        );
//         log::info!("timeout: {:?}", &timeout);
        let timeout = sleep(timeout);
        
        let f = async {
        let mut ctx = self.ctx.lock().await;
        for r in ranges_address {
            let buff = {
                
                let buff = ctx.read_holding_registers(*r.start(), *r.end() - *r.start()+1);
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            
                buff.await
            };
            if let Ok(buff) = buff {
                Self::update_impl(&self.values, r.clone(), buff);
            } else {
                log::error!("buff: {:?}", buff);
                log::error!("Range ({:?})\nRanges: {:?}", r, &ranges_address);
                return Err(DeviceError::ValueOut);
            }
        }
        Ok(())
        };
        
        tokio::select! {
        res = f => res,
        _ = timeout => Err(DeviceError::TimeOut),
        }
    }
    
    pub(crate) async fn set_value(&self, v: &Value) -> Result<(), DeviceError> {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::Writer;
        match v.size.size() {
        1 => self.ctx.lock().await.write_single_register(v.address(), v.value() as u16).await?,
        2 => {
            self.ctx.lock().await.write_single_register(v.address(), v.value() as u16).await?;
            self.ctx.lock().await.write_single_register(v.address()+1, (v.value()>>16) as u16).await?;
        },
        _ => {}
        };
        Ok(())
    }
}
