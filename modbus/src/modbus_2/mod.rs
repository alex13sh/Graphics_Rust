mod init;

mod value;
mod sensor;
mod device;
use value::*;
use sensor::*;
use device::*;

// impl From<init::Value> for Value {
//     fn from(v: init::Value) -> Value {
//         Value {
//             name: v.name,
//             
//         }
//     }
// }



// #[test]
pub fn tst() {

    init::tst();
}
