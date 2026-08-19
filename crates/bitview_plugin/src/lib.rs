#![doc = include_str!("../README.md")]

mod compute_plugin;
mod gate;
mod plugin;
mod plugin_data;
mod plugin_id;
mod read_guard;

pub use compute_plugin::ComputePlugin;
pub use gate::PluginGate;
pub use plugin::Plugin;
pub use plugin_data::PluginData;
pub use plugin_id::PluginId;
pub use read_guard::PluginReadGuard;

/// Directory containing one directory per active plugin below the Bitview data root.
pub const PLUGIN_DATA_DIR: &str = "plugins";
