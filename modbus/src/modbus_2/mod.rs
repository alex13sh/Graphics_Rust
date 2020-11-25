#![allow(dead_code)]

pub mod init;
// use init::*;

pub mod value;
pub mod sensor;
pub mod device;
pub mod invertor;
pub mod owen_digit_io;

pub use value::*;
pub use sensor::*;
pub use device::*;
pub use invertor::*;
pub use owen_digit_io::DigitIO;

use tokio_modbus::client::sync::Context as ModbusContext;

// #[test]
pub(crate) fn tst() {

//     init::tst();
    let d = init::init_devices();
    let devices: Vec<_> = d.into_iter().map(|d| Device::from(d)).collect();
    dbg!(&devices);
    for d in devices {
        let r = d.get_ranges_value(8, true);
        dbg!(r);
    }

}

