mod dependencies;
mod has;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasCapitalSentiment;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("capital_sentiment");
