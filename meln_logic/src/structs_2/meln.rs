#![allow(dead_code)]

use modbus::ModbusValues;
use super::HalfMeln;
use super::Dozator;
use super::OilStation;

pub struct Meln {
    dozator: Dozator,
    half_top: HalfMeln,
    half_button: HalfMeln,
    
    oil: OilStation,
}

impl From<&ModbusValues> for Meln {
    fn from(values: &ModbusValues) -> Self {
        Meln {
            dozator: values.into(),
            half_top: HalfMeln::top(values),
            half_button: HalfMeln::low(values),
            
            oil: values.into(),
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
