mod meln;
mod half_meln;
mod dozator;
#[macro_use]
mod property;
mod oil_station;
mod vacuum_station;
mod material;

use meln::Meln as MelnValues;
use half_meln::HalfMeln;
use dozator::Dozator;
pub use property::Property;
use oil_station::OilStation;
use vacuum_station::VacuumStation;
use material::Material;

pub use meln::watcher::Meln as MelnWatch;
use property::{changed_all, changed_any};
use std::sync::Arc;

#[derive(Clone)]
pub struct Meln {
    pub values: Arc<MelnValues>,
    pub properties: Arc<MelnWatch>,
}

impl Meln {
    pub fn new(values: &modbus::ModbusValues) -> Self {
        Meln {
            values: Arc::new(values.into()),
            properties: Default::default(),
        }
    }
    pub async fn automation(&self) {
        self.properties.automation().await
    }
    pub async fn automation_mut(&self) {
        meln::watcher::automation_mut(&self.values, &self.properties).await
    }
}
