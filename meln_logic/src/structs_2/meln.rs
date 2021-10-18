#![allow(dead_code)]

use modbus::ModbusValues;
use super::HalfMeln;
use super::Dozator;
use super::OilStation;
use super::VacuumStation;
use super::Material;

pub struct Meln {
    
    pub material: Material,
    
    pub half_top: HalfMeln,
    pub half_bottom: HalfMeln,
    
    pub oil: OilStation,
    pub vacuum: VacuumStation,
}

impl From<&ModbusValues> for Meln {
    fn from(values: &ModbusValues) -> Self {
        Meln {
            material: values.into(),
            
            half_top: HalfMeln::top(values),
            half_bottom: HalfMeln::low(values),
            
            oil: values.into(),
            vacuum: values.into(),
        }
    }
}

pub mod watcher {
    use super::super::*;
    use half_meln::watcher::HalfMeln;
    use oil_station::watcher::OilStation;
    use vacuum_station::watcher::VacuumStation;
    use material::watcher::Material;
    
    pub struct Meln {
        pub material: Material,
        
        pub half_top: HalfMeln,
        pub half_bottom: HalfMeln,
        pub is_started: Property<bool>,
        
        pub oil: OilStation,
        pub vacuum: VacuumStation,
    }
    
    impl Meln {
        fn update_property(&self, values: &super::Meln) {
            self.material.update_property(&values.material);
            
            self.half_top.update_property(&values.half_top);
            self.half_bottom.update_property(&values.half_bottom);
            
            self.oil.update_property(&values.oil);
            self.vacuum.update_property(&values.vacuum);
        }
        
        pub async fn automation(&self) {
            let is_started = async {
                let mut start_top = self.half_top.is_started.subscribe();
                let mut start_bottom = self.half_bottom.is_started.subscribe();
                
                loop {
                    crate::changed_any!(start_top, start_bottom);
                    let start_top = *start_top.borrow();
                    let start_bottom = *start_bottom.borrow();
                    
                    self.is_started.set(start_top || start_bottom);
                }
            };
            tokio::join!(
                is_started,
                self.half_top.automation(),
                self.half_bottom.automation(),
            );
        }
    }
    }
}
