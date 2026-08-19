mod breakdown;
mod by_kind;
mod dependencies;
mod has;
mod policy;
mod total;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasOpReturn;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("op_return");
