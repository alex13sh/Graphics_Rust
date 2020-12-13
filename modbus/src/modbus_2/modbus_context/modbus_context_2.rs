use libmodbus_rs::{Modbus, ModbusClient, ModbusTCP, Timeout};

use super::{Value, ModbusValues};
use super::init::DeviceAddress;
use std::collections::HashMap;
use std::sync::Arc;

use super::device::{
    DeviceError,
    get_ranges_value, convert_modbusvalues_to_hashmap_address,
};

pub(super) struct ModbusContext {
    ctx: Modbus,
    pub(super) values: HashMap<u16, Arc<Value>>,
    ranges_address: Vec<std::ops::RangeInclusive<u16>>,
}

impl ModbusContext {
    pub fn new(address: &DeviceAddress, values: &ModbusValues) -> Option<Self> {
        if cfg!(not(feature = "test")) {
        if let DeviceAddress::TcpIP(txt) = address {
            let mut modbus = Modbus::new_tcp(txt, 502).ok()?;
            modbus.set_debug(true).ok()?;
            modbus.connect().ok()?;
            Some(ModbusContext {
                ctx: modbus,
                ranges_address: get_ranges_value(&values, 8, true),
                values: convert_modbusvalues_to_hashmap_address(values),
            })
        } else {
            None
        }
        } else {None}
    }
    pub fn update(&mut self) -> Result<(), DeviceError> {
        for r in &self.ranges_address {
            let cnt = (*r.end() - *r.start()+1);
            let mut buff = vec![0u16; cnt as usize];
            self.ctx.read_registers(*r.start(), cnt, &mut buff)?;
//             println!("Ranges ({:?}) is '{:?}'", r, buff);
            let itr_buff = buff.into_iter();
            let mut itr_zip = r.clone().zip(itr_buff);
            while let Some((adr, v)) = itr_zip.next() {
//             for (adr, v) in itr_zip {
                if let Some(val) = self.values.get_mut(&adr) {
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
        Ok(())
    }
    
    pub(super) fn set_value(&mut self, _v: &Value) -> Result<(), DeviceError> {
//         let v = self.values.get(address).unwrap().clone();
//         match v.size.size() {
//         1 => self.ctx.write_single_register(v.address(), v.value() as u16)?,
//         2 => {
//             self.ctx.write_single_register(v.address(), v.value() as u16)?;
//             self.ctx.write_single_register(v.address()+1, (v.value()>>16) as u16)?;
//         },
//         _ => {}
//         };
        Ok(())
    }
    pub(super) fn get_value(&mut self, _v: &Value) -> Result<(), DeviceError>  {
//         let v = self.values.get(address).unwrap().clone();
//         use tokio_modbus::client::sync::Reader;
//         match v.size.size() {
//         1 => v.update_value(self.ctx.read_holding_registers(v.address(), 1)?[0] as u32),
//         2 => {
//             let buf = self.ctx.read_holding_registers(v.address(), 2)?;
//             v.update_value((buf[0] as u32) | (buf[1] as u32)<<16);
//         },
//         _ => {}
//         };
        Ok(())
    }
}

impl From<libmodbus_rs::prelude::Error> for DeviceError {
    fn from(_err: libmodbus_rs::prelude::Error) -> Self {
        DeviceError::ValueError
    }
}
