use tokio_modbus::client::sync::Context;

use super::{Value, ModbusValues, ModbusSensors};
use super::init::{DeviceType, DeviceAddress};
use std::collections::HashMap;
use std::sync::Arc;

use super::device::DeviceError;

pub(super) struct ModbusContext {
    ctx: Context,
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
                ranges_address: super::device::get_ranges_value(&values, 8, true),
                values: super::device::convert_modbusvalues_to_hashmap_address(values),
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
