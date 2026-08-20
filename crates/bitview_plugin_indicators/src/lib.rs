mod dependencies;
mod dormancy_vecs;
mod gini;
mod has;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasIndicators;
pub use vecs::Vecs;

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("indicators"), Version::new(10));
pub const ID: PluginId = STORAGE.id();
