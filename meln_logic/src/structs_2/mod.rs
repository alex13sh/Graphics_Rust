mod meln;
mod half_meln;
mod dozator;
#[macro_use]
mod property;
mod oil_station;
mod vacuum_station;
mod material;
mod klapans;

use meln::Meln as MelnValues;
use half_meln::HalfMeln;
use dozator::Dozator;
use oil_station::OilStation;
use vacuum_station::VacuumStation;
use material::Material;
use klapans::Klapans;

pub use meln::watcher::Meln as MelnWatch;
pub use property::Property;
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

pub mod values {
    use super::*;
    pub use meln::Meln;
    pub use half_meln::{
        HalfMeln, HalfPart,
        Invertor,
    };
    pub use dozator::Dozator;
    pub use oil_station::OilStation;
    pub use vacuum_station::VacuumStation;
    pub use material::Material;
    pub use klapans::Klapans;
}

pub mod watcher {
    use super::*;
    pub use meln::watcher::{Meln, MelnStep};
    pub use half_meln::watcher::HalfMeln;
    pub use dozator::watcher::Dozator;
    pub use oil_station::watcher::OilStation;
    pub use vacuum_station::watcher::VacuumStation;
    pub use material::watcher::Material;
    pub use klapans::watcher::Klapans;
    
    pub use property::Property;
    pub use tokio::sync::watch::Receiver as Subscription;
}
