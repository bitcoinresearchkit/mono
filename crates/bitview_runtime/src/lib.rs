#![doc = include_str!("../README.md")]

mod bootstrap;
mod plugin_set;
mod update;

pub use bitview_plugin::{DEFAULT_CACHE_BUDGET, ImportContext, UpdateContext};
pub use bitview_runtime_derive::PluginSet;
pub use bootstrap::bootstrap;
pub use plugin_set::{BootstrapAction, ComputePluginSet, PluginSet};
pub use update::update;
