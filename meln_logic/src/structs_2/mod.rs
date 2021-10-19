mod meln;
mod half_meln;
mod dozator;
#[macro_use]
mod property;
mod oil_station;
mod vacuum_station;
mod material;

use meln::Meln;
use half_meln::HalfMeln;
use dozator::Dozator;
pub use property::Property;
use oil_station::OilStation;
use vacuum_station::VacuumStation;
use material::Material;

pub use meln::watcher::Meln as MelnWatch;
use property::{changed_all, changed_any};

pub fn create_meln(values: &modbus::ModbusValues) -> (Meln, MelnWatch) {
    (
        values.into(),
        Default::default(),
    )
}
