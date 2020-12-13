#[cfg(feature = "tokio-modbus")]
mod modbus_context_1;
#[cfg(feature = "libmodbus-rs")]
mod modbus_context_2;
#[cfg(feature = "modbus-rs")]
mod modbus_context_3;

#[cfg(feature = "tokio-modbus")]
pub(super) use modbus_context_1::ModbusContext;
#[cfg(feature = "libmodbus-rs")]
pub(super) use modbus_context_2::ModbusContext;
#[cfg(feature = "modbus-rs")]
pub(super) use modbus_context_3::ModbusContext;

use super::*;
