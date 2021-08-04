pub mod invertor;
pub use invertor::Invertor;
pub mod klapans_1;
pub mod klapans_2;
pub use klapans_2 as klapans;
pub use klapans::Klapans;
pub mod dozator;
pub use dozator::Dozator;
#[macro_use]
pub mod values_list;
pub use values_list::{ValuesList, make_value_lists, make_value_lists_start};

pub mod oil_station;
pub use oil_station::OilStation;

pub mod style;

mod liner_animation;

pub mod animations {
    pub use super::liner_animation::LinerAnimation;

    #[derive(Debug, Clone)]
    pub enum Progress {
        Value(f32),
        Finished,
    }
}
