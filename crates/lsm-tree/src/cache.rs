// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use quick_cache::{
    OptionsBuilder, Weighter,
    sync::{Cache as QuickCache, DefaultLifecycle},
};
use rustc_hash::FxBuildHasher;

use crate::{
    GlobalTableId,
    table::{
        Block, BlockOffset,
        block::{BlockType, Header},
    },
};

#[derive(Eq, std::hash::Hash, PartialEq)]
struct CacheKey(GlobalTableId, u64);

impl CacheKey {
    fn from_id(id: GlobalTableId, offset: BlockOffset) -> Self {
        Self(id, *offset)
    }
}

#[derive(Clone)]
struct BlockWeighter;

impl Weighter<CacheKey, Block> for BlockWeighter {
    fn weight(&self, _: &CacheKey, block: &Block) -> u64 {
        (Header::serialized_len() as u64) + u64::from(block.header.uncompressed_length)
    }
}

type BlockCache = QuickCache<CacheKey, Block, BlockWeighter, FxBuildHasher>;

/// Shared cache for decoded table blocks.
///
/// Indexes and filters receive seven eighths of the capacity. The remaining
/// eighth keeps reused data blocks without letting random data reads evict
/// the metadata needed by many lookups.
///
/// # Examples
///
/// Sharing cache between multiple trees
///
/// ```
/// # use lsm_tree::{Tree, Config, Cache};
/// # use std::sync::Arc;
/// #
/// // Provide 64 MB of cache capacity
/// let cache = Arc::new(Cache::with_capacity_bytes(64 * 1_000 * 1_000));
///
/// # let folder = tempfile::tempdir()?;
/// let tree1 = Tree::open(Config::new(folder.path()).use_cache(cache.clone()))?;
/// # let folder = tempfile::tempdir()?;
/// let tree2 = Tree::open(Config::new(folder.path()).use_cache(cache.clone()))?;
/// #
/// # Ok::<(), lsm_tree::Error>(())
/// ```
pub struct Cache {
    // NOTE: rustc_hash performed best: https://fjall-rs.github.io/post/fjall-2-1
    data: BlockCache,
    metadata: BlockCache,

    /// Capacity in bytes
    capacity: u64,
}

impl Cache {
    /// Creates a block cache with roughly `bytes` bytes of capacity.
    #[must_use]
    pub fn with_capacity_bytes(bytes: u64) -> Self {
        let data_bytes = bytes / 8;
        Self {
            data: Self::create_cache(data_bytes),
            metadata: Self::create_cache(bytes - data_bytes),
            capacity: bytes,
        }
    }

    fn create_cache(bytes: u64) -> BlockCache {
        #[expect(clippy::expect_used, reason = "nothing we can do if it fails")]
        let opts = OptionsBuilder::new()
            .weight_capacity(bytes)
            .hot_allocation(0.8)
            .estimated_items_capacity(10_000)
            .build()
            .expect("cache options should be valid");

        QuickCache::with_options(
            opts,
            BlockWeighter,
            FxBuildHasher,
            DefaultLifecycle::default(),
        )
    }

    /// Returns the amount of cached bytes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.data.weight() + self.metadata.weight()
    }

    /// Returns the cache capacity in bytes.
    #[must_use]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    #[must_use]
    pub(crate) fn get_block(
        &self,
        id: GlobalTableId,
        offset: BlockOffset,
        block_type: BlockType,
    ) -> Option<Block> {
        let key = CacheKey::from_id(id, offset);
        self.cache_for(block_type).get(&key)
    }

    fn cache_for(&self, block_type: BlockType) -> &BlockCache {
        if block_type == BlockType::Data {
            &self.data
        } else {
            &self.metadata
        }
    }

    pub(crate) fn insert_block(&self, id: GlobalTableId, offset: BlockOffset, block: Block) {
        self.cache_for(block.header.block_type)
            .insert(CacheKey::from_id(id, offset), block);
    }
}
