#![allow(dead_code)]

use modbus::ModbusValues;
use super::HalfMeln;
use super::Dozator;
use super::OilStation;
use super::VacuumStation;
use super::Material;

pub struct Meln {
    dozator: Dozator,
    material: Material,
    
    half_top: HalfMeln,
    half_button: HalfMeln,
    
    oil: OilStation,
    vacuum: VacuumStation,
}

impl From<&ModbusValues> for Meln {
    fn from(values: &ModbusValues) -> Self {
        Meln {
            dozator: values.into(),
            material: values.into(),
            
            half_top: HalfMeln::top(values),
            half_button: HalfMeln::low(values),
            
            oil: values.into(),
            vacuum: values.into(),
        }
    }
}

pub mod watcher {
    use super::super::*;
    use half_meln::watcher::HalfMeln;
    use dozator::watcher::Dozator;
    use oil_station::watcher::OilStation;
    use vacuum_station::watcher::VacuumStation;
    use material::watcher::Material;
    
    pub struct Meln {
        dozator: Dozator,
        material: Material,
        
        half_top: HalfMeln,
        half_button: HalfMeln,
        
        oil: OilStation,
        vacuum: VacuumStation,
    }
    
    impl Meln {
        fn update_property(&self, values: &super::Meln) {
            self.dozator.update_property(&values.dozator);
            self.material.update_property(&values.material);
            
            self.half_top.update_property(&values.half_top);
            self.half_button.update_property(&values.half_button);
            
            self.oil.update_property(&values.oil);
            self.vacuum.update_property(&values.vacuum);
        }
    }
}
