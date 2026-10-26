mod compute;
mod dependencies;
mod has;
mod import;
mod pool;
mod pool_heights;

pub use dependencies::Dependencies;
pub use has::HasPools;
pub use pool::PoolVecs;

use std::{collections::BTreeMap, ops::Deref};

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{PoolSlug, Pools};
use bitview_traversable::{AnyExportableVec, AnyStoredVec, Traversable, TreeNode};
use brk_types::Height;
use vecdb::{BytesVec, Database, Rw, StorageMode, Version};

use pool_heights::PoolHeights;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("pools"), Version::new(13));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    pools: M::WriteOnly<&'static Pools>,

    /// Mining pool attributed to each block: the first pool in the known-pool list that matches
    /// either a payout address among the coinbase transaction's outputs or, by case-insensitive
    /// substring, the coinbase tag (mempool.space's order). Unmatched blocks are `unknown`.
    #[traversable(lead = "Which mining pool mined each block.")]
    pub block_pool: M::Stored<BytesVec<Height, PoolSlug>>,
    #[traversable(skip)]
    pub heights: PoolHeights,
    // Each known mining pool, plus `unknown` for unattributed blocks.
    #[traversable(flatten)]
    pub by_slug: BySlug,
}

/// Each pool's series, by its slug: their descriptions led by the pool they're of (see `Pool::lead`).
#[derive(Clone)]
pub struct BySlug(BTreeMap<PoolSlug, PoolVecs>);

impl Deref for BySlug {
    type Target = BTreeMap<PoolSlug, PoolVecs>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Traversable for BySlug {
    fn to_tree_node(&self) -> TreeNode {
        self.0.to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.0.iter_any_exportable()
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.0.iter_any_visible()
    }

    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.0.iter_any_stored_mut()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<bitview_traversable::Fragment>,
        descriptions: &mut BTreeMap<&'a str, Vec<bitview_traversable::Fragment>>,
    ) {
        let pools = bitview_primitives::pools();
        for (slug, vecs) in &self.0 {
            let lead = bitview_traversable::Fragment::Lead(pools.get(*slug).lead);
            description_fragments.push(lead);
            vecs.collect_series_descriptions(description_fragments, descriptions);
            let popped = description_fragments.pop();
            debug_assert_eq!(popped, Some(lead));
        }
    }
}
