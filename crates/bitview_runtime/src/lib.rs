#![doc = include_str!("../README.md")]

extern crate self as bitview_runtime;

mod bootstrap;
mod plugin_set;
mod update;

pub use bitview_plugin::{ImportContext, UpdateContext};
pub use bitview_runtime_derive::PluginSet;
pub use bootstrap::bootstrap;
pub use plugin_set::{BootstrapAction, ComputePluginSet, PluginSet};
pub use update::update;
