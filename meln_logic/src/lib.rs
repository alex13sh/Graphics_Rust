
#[cfg(feature = "init")]
pub mod init;

#[cfg(feature = "init_clear")]
pub mod init_clear;
#[cfg(feature = "init_clear")]
pub use init_clear::init;

// pub mod algorithm;
pub mod devices;

mod structs_2;
use structs_2::*;
