#![allow(dead_code)]

use super::HalfMeln;
use super::Dozator;
use modbus::ModbusValues;

pub struct Meln {
    dozator: Dozator,
    half_top: HalfMeln,
    half_button: HalfMeln,
}

impl From<&ModbusValues> for Meln {
    fn from(values: &ModbusValues) -> Self {
        Meln {
            dozator: values.into(),
            half_top: HalfMeln::top(values),
            half_button: HalfMeln::low(values),
        }
    }
}

pub mod watcher {
    use super::super::*;
    use half_meln::watcher::HalfMeln;
    use dozator::watcher::Dozator;
    
    pub struct Meln {
        dozator: Dozator,
        half_top: HalfMeln,
        half_button: HalfMeln,
    }
    
    impl Meln {
        fn update_property(&self, values: &super::Meln) {
            self.dozator.update_property(&values.dozator);
            self.half_top.update_property(&values.half_top);
            self.half_button.update_property(&values.half_button);
        }
    }
}
