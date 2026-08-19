mod dependencies;
mod dormancy_vecs;
mod gini;
mod vecs;

pub use dependencies::Dependencies;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("indicators");
const DB_NAME: &str = ID.as_str();
