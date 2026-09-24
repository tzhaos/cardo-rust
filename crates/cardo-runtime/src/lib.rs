#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "diagnostics")]
pub mod diagnostics;
#[cfg(feature = "localization")]
pub mod localization;
#[cfg(windows)]
pub mod registry;
pub mod storage;

#[cfg(feature = "localization")]
pub mod messages;
pub mod settings;
pub mod shortcuts;
pub mod task;
