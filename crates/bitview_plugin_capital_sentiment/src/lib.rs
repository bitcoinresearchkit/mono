mod dependencies;
mod has;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasCapitalSentiment;
pub use vecs::Vecs;

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("capital_sentiment"), Version::new(14));
pub const ID: PluginId = STORAGE.id();
