mod init;

mod value;
mod sensor;
mod device;
use value::*;
use sensor::*;
use device::*;


// #[test]
pub fn tst() {

//     init::tst();
    let d = init::init_devices();
    let d: Vec<_> = d.into_iter().map(|d| Device::from(d)).collect();
    dbg!(d);
}
