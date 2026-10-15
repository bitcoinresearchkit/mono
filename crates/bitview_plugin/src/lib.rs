#![doc = include_str!("../README.md")]

mod compute_plugin;
mod import_context;
mod plugin;
mod plugin_data;
mod plugin_id;
mod plugin_set;
mod storage;
mod update_context;

#[cfg(feature = "derive")]
pub use bitview_plugin_derive::{Plugin, PluginSet};
pub use compute_plugin::ComputePlugin;
pub use import_context::ImportContext;
pub use plugin::Plugin;
pub use plugin_data::PluginData;
pub use plugin_id::PluginId;
pub use plugin_set::{BootstrapAction, ComputePluginSet, PluginSet};
pub use storage::PluginStorage;
pub use update_context::UpdateContext;

/// The [`ComputePlugin`] items every database-backed plugin repeats: its `Dependencies` type and
/// the database its computation syncs and compacts (`self.$db`).
#[macro_export]
macro_rules! compute_items {
    ($dependencies:ident, $db:ident) => {
        type Dependencies<'a> = $dependencies<'a>;

        fn database(&self) -> Option<&::vecdb::Database> {
            Some(&self.$db)
        }
    };
}

/// Directory containing one directory per active plugin below the Bitview data root.
pub(crate) const PLUGIN_DATA_DIR: &str = "plugins";
