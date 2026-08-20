mod burned;
mod dependencies;
mod has;
mod velocity;

mod vecs;

pub use dependencies::Dependencies;
pub use has::HasSupply;
pub use vecs::Vecs;

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("supply"), Version::new(10));
pub const ID: PluginId = STORAGE.id();
