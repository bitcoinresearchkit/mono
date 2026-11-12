mod antpool_group;
mod compute;
mod concentration;
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
use bitview_primitives::{Count, PoolSlug, Pools};
use bitview_traversable::{AnyExportableVec, AnyStoredVec, Traversable, TreeNode};
use bitview_vecs::CumulativeRolling;
use brk_types::Height;
use vecdb::{BytesVec, Database, Rw, StorageMode, Version};

use concentration::Concentration;
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
    /// What the largest pools of each day found, ranked by that day's blocks.
    concentration: Concentration<M>,
    /// The same, with AntPool and the pools sharing its block templates counted as one.
    concentration_grouped: Concentration<M>,
    /// Blocks found by AntPool and the pools sharing its block templates: Poolin, Ultimus, SECPOOL,
    /// Sigmapool, Binance Pool, Rawpool, Luxor, CloverPool and Mining Squared, and Braiins below
    /// height 956,057.
    #[traversable(
        wrap = "groups/antpool",
        rename = "block_count",
        lead = "Blocks found by AntPool and the pools sharing its block templates."
    )]
    antpool_group: CumulativeRolling<Count, M>,
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

    fn collect_series_metadata<'a>(
        &'a self,
        path_fragments: &mut Vec<bitview_traversable::MetadataFragment>,
        metadata: &mut BTreeMap<&'a str, Vec<bitview_traversable::MetadataFragment>>,
    ) {
        let pools = bitview_primitives::pools();
        for (slug, vecs) in &self.0 {
            let lead = bitview_traversable::MetadataFragment::Description(
                bitview_traversable::Fragment::Lead(pools.get(*slug).lead),
            );
            path_fragments.push(lead);
            vecs.collect_series_metadata(path_fragments, metadata);
            let popped = path_fragments.pop();
            debug_assert_eq!(popped, Some(lead));
        }
    }
}
