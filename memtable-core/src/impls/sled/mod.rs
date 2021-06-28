#[cfg(not(feature = "serde-1"))]
compile_error!("sled requires serde to be enabled");

mod dynamic;
// mod fixed;

pub use dynamic::*;
// pub use fixed::*;
