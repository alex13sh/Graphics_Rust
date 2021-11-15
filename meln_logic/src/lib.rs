pub mod devices;
pub use devices as init;

pub mod meln;
pub use meln::{Meln as MelnValues};
pub mod watcher;
use watcher::Meln as MelnWatch;

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
