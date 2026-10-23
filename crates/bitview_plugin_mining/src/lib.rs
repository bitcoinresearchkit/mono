mod compute;
mod dependencies;
mod has;
mod hash;
mod import;
mod rewards;

pub use dependencies::Dependencies;
pub use has::HasMining;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use hash::Vecs as HashVecs;
use rewards::Vecs as RewardsVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mining"), Version::new(11));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    pub rewards: RewardsVecs<M>,
    #[traversable(flatten)]
    pub hash: HashVecs<M>,
}
