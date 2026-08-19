#![doc = include_str!("../README.md")]

mod bootstrap;
mod plugin_set;
mod update;

pub use bootstrap::bootstrap;
pub use plugin_set::{BootstrapAction, ComputePluginSet, PluginSet};
pub use update::update;
