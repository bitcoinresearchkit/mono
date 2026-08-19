mod burned;
mod dependencies;
mod velocity;

mod vecs;

pub use dependencies::Dependencies;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("supply");
const DB_NAME: &str = ID.as_str();
