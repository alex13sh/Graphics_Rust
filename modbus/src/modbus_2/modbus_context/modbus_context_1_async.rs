use tokio_modbus::client::Context;
use futures::executor::block_on;

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
    ctx: Arc<Mutex<Context>>,
//     ctx: Context,
    pub(crate) values: Values,
    ranges_address: Vec<RangeAddress>,
}

impl ModbusContext {
    
    pub async fn new(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        if cfg!(not(feature = "test")) {
        if let DeviceAddress::TcpIP(txt) = address {
            use tokio_modbus::prelude::*;
            let socket_addr = (txt.to_owned()+":502").parse().ok()?;
            dbg!(&socket_addr);
            
            Some(ModbusContext {
                ctx: Arc::new(Mutex::new(tcp::connect(socket_addr).await.ok()?)),
//                 ctx: tcp::connect(socket_addr).await.ok()?,
                ranges_address: get_ranges_value(&values, 8, true),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } else {
            None
        }
        } else {None}
    }
    
    pub async fn new_async(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        Self::new(address, values).await
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
        use tokio_modbus::client::Reader;
        for r in &self.ranges_address {
            let buff = block_on(self.ctx.lock()?.read_holding_registers(*r.start(), *r.end() - *r.start()+1))?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        Ok(())
    }
    
    pub async fn update_async(&self) -> Result<(), DeviceError> {
        use tokio_modbus::client::Reader;
        for r in &self.ranges_address {
            let buff = self.ctx.lock()?.read_holding_registers(*r.start(), *r.end() - *r.start()+1).await?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            Self::update_impl(&self.values, r.clone(), buff);
        }
        Ok(())
    }
    
    pub(crate) fn is_busy(&self) -> bool {
        self.ctx.is_poisoned()
    }
    
    pub(crate) fn set_value(&self, v: &Value) -> Result<(), DeviceError> {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::Writer;
        match v.size.size() {
        1 => block_on(self.ctx.lock()?.write_single_register(v.address(), v.value() as u16))?,
        2 => {
            block_on(self.ctx.lock()?.write_single_register(v.address(), v.value() as u16))?;
            block_on(self.ctx.lock()?.write_single_register(v.address()+1, (v.value()>>16) as u16))?;
        },
        _ => {}
        };
        Ok(())
    }
    pub(crate) fn get_value(&self, v: &Value) -> Result<(), DeviceError>  {
//         let v = self.values.get(address).unwrap().clone();
        use tokio_modbus::client::Reader;
        match v.size.size() {
        1 => v.update_value(block_on(self.ctx.lock()?.read_holding_registers(v.address(), 1))?[0] as u32),
        2 => {
            let buf = block_on(self.ctx.lock()?.read_holding_registers(v.address(), 2))?;
            v.update_value((buf[0] as u32) | (buf[1] as u32)<<16);
        },
        _ => {}
        };
        Ok(())
    }
}
