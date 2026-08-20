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

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("op_return"), Version::new(9));
pub const ID: PluginId = STORAGE.id();
