#![doc = include_str!("../README.md")]

mod compute_plugin;
mod import_context;
mod plugin;
mod plugin_data;
mod plugin_id;
mod publication;
mod storage;
mod update_context;

pub use compute_plugin::ComputePlugin;
pub use import_context::ImportContext;
pub use plugin::Plugin;
pub use plugin_data::PluginData;
pub use plugin_id::PluginId;
pub use publication::{Publication, PublicationReadGuard};
pub use storage::PluginStorage;
pub use update_context::UpdateContext;

/// Default size, in bytes, of the shared vector cache (`vecdb::Budgeted::init_global`).
pub const DEFAULT_CACHE_BUDGET: usize = 2 << 30;

/// Directory containing one directory per active plugin below the Bitview data root.
pub(crate) const PLUGIN_DATA_DIR: &str = "plugins";
