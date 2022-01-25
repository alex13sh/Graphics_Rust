#[macro_use] 
mod property;
pub use property::Property;
pub(crate) use property::{changed_all, changed_any};

use super::meln;
pub use meln::watcher::{Meln, MelnStep};
pub use meln::{
    half_meln::watcher::HalfMeln,
    dozator::watcher::Dozator,
    oil_station::watcher::OilStation,
    vacuum_station::watcher::{VacuumStation, Status as VacuumStatus},
    material::watcher::Material,
    klapans::watcher::Klapans,
};

pub use tokio::sync::watch::Receiver as Subscription;
