#![allow(dead_code)]

pub mod init;
// use init::*;

pub mod value;
pub mod device;
pub mod devices;

pub use value::*;
pub use init::ValueError;
pub use device::*;
pub use devices::*;

mod modbus_context;
use modbus_context::ModbusContext;

// #[test]
pub fn tst() {

//     init::tst();
    let d = init::init_devices();
    let devices: Vec<_> = d.into_iter().map(|d| Device::from(d)).collect();
    dbg!(&devices);
//     for d in devices {
//         let r = d.get_ranges_value(8, true);
//         dbg!(r);
//     }

}

