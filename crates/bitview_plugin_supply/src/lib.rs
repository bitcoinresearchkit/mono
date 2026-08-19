mod burned;
mod dependencies;
mod has;
mod velocity;

mod vecs;

pub use dependencies::Dependencies;
pub use has::HasSupply;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("supply");
