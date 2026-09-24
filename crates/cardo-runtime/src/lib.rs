#[cfg(feature = "diagnostics")]
pub mod diagnostics;
#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "localization")]
pub mod localization;
#[cfg(windows)]
pub mod registry;
pub mod storage;

pub mod task;
pub mod shortcuts;
pub mod settings;
