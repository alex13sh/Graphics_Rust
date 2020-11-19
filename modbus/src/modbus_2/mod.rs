#![allow(dead_code)]

mod init;

mod value;
mod sensor;
mod device;
mod invertor;

use value::*;
use sensor::*;
use device::*;
use invertor::*;

use tokio_modbus::client::sync::Context as ModbusContext;

// #[test]
pub fn tst() {

//     init::tst();
    let d = init::init_devices();
    let mut devices: Vec<_> = d.into_iter().map(|d| Device::from(d)).collect();
//     dbg!(&devices);
    for d in devices {
        let r = d.get_ranges_value(8, true);
        dbg!(r);
    }
}
