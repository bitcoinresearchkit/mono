mod compute;
mod dependencies;
mod has;
mod import;
mod pool;
mod pool_heights;

pub use dependencies::Dependencies;
pub use has::HasPools;
pub use pool::PoolVecs;

use std::collections::BTreeMap;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{PoolSlug, Pools};
use bitview_traversable::Traversable;
use brk_types::Height;
use vecdb::{BytesVec, Database, Rw, StorageMode, Version};

use pool_heights::PoolHeights;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("pools"), Version::new(13));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    pools: M::WriteOnly<&'static Pools>,

    /// Mining pool attributed to each block. BRK first scans address-bearing
    /// outputs of the coinbase transaction for a known pool payout address; if
    /// none matches, it performs case-insensitive substring matching against
    /// known coinbase tags. Unmatched blocks are classified as `unknown`.
    pub pool: M::Stored<BytesVec<Height, PoolSlug>>,
    #[traversable(skip)]
    pub heights: PoolHeights,
    // Each known mining pool, plus `unknown` for unattributed blocks.
    #[traversable(flatten)]
    pub by_slug: BTreeMap<PoolSlug, PoolVecs>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
