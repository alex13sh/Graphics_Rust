pub mod invertor;
pub use invertor::Invertor;

pub mod klapans;
pub use klapans::Klapans;
pub mod dozator;
pub use dozator::Dozator;
#[macro_use]
pub mod values_list;
pub use values_list::{ValuesList, make_value_lists, make_value_lists_start};

pub mod oil_station;
pub use oil_station::OilStation;
pub mod info_pane;
pub use info_pane::InfoPane;

pub mod style;

mod property_animation;

pub mod animations {
    pub use super::property_animation::{PropertyAnimation, BroadcastAnimation};

    #[derive(Debug, Clone)]
    pub enum Progress {
        Value(f32),
        Finished,
    }
}
