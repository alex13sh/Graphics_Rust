
#[cfg(feature = "init")]
pub mod init;

#[cfg(feature = "init_clear")]
pub mod init_clear;
#[cfg(feature = "init_clear")]
pub use init_clear::init;

// pub mod algorithm;
pub mod devices;

pub mod structs_2;
pub use structs_2 as structs;
pub use structs_2::create_meln;
