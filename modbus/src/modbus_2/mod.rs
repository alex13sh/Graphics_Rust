mod init;

mod value;
mod sensor;
mod device;
use value::*;
use sensor::*;
use device::*;

use tokio_modbus::client::Context as ModbusContext;

// #[test]
pub fn tst() {

//     init::tst();
    let d = init::init_devices();
    let mut devices: Vec<_> = d.into_iter().map(|d| Device::from(d)).collect();
    dbg!(&devices);
    
    use tokio_modbus::prelude::*;
    for d in &mut devices {
        let socket_addr = match &d.name()[..] {
        "Input Analog" => "192.168.0.1:502".parse().unwrap(),
        "Input/Output Digit" => "192.168.0.2:502".parse().unwrap(),
        "Invertor" => "192.168.0.3:502".parse().unwrap(),
        _ => "192.168.0.1:502".parse().unwrap(),
        };
        let mut ctx = sync::tcp::connect(socket_addr).unwrap();
    }
}
