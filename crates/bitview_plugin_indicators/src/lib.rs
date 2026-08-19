mod dependencies;
mod dormancy_vecs;
mod gini;
mod has;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasIndicators;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("indicators");
