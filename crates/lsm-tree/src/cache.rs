// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::GlobalTableId;
use crate::table::block::Header;
use crate::table::{Block, BlockOffset};
use quick_cache::Weighter;
use quick_cache::sync::Cache as QuickCache;

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

/// Cache in which table blocks are cached in memory
/// after being retrieved from disk
///
/// This speeds up consecutive queries to nearby data, improving
/// read performance for hot data.
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
    /// Concurrent cache implementation
    data: QuickCache<CacheKey, Block, BlockWeighter, rustc_hash::FxBuildHasher>,

    /// Capacity in bytes
    capacity: u64,
}

impl Cache {
    /// Creates a new block cache with roughly `n` bytes of capacity.
    #[must_use]
    pub fn with_capacity_bytes(bytes: u64) -> Self {
        use quick_cache::sync::DefaultLifecycle;

        #[expect(clippy::expect_used, reason = "nothing we can do if it fails")]
        let opts = quick_cache::OptionsBuilder::new()
            .weight_capacity(bytes)
            .hot_allocation(0.8)
            .estimated_items_capacity(10_000)
            .build()
            .expect("cache options should be valid");

        let quick_cache = QuickCache::with_options(
            opts,
            BlockWeighter,
            rustc_hash::FxBuildHasher,
            DefaultLifecycle::default(),
        );

        Self {
            data: quick_cache,
            capacity: bytes,
        }
    }

    /// Returns the amount of cached bytes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.data.weight()
    }

    /// Returns the cache capacity in bytes.
    #[must_use]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    #[doc(hidden)]
    #[must_use]
    pub fn get_block(&self, id: GlobalTableId, offset: BlockOffset) -> Option<Block> {
        let key = CacheKey::from_id(id, offset);
        self.data.get(&key)
    }

    #[doc(hidden)]
    pub fn insert_block(&self, id: GlobalTableId, offset: BlockOffset, block: Block) {
        self.data.insert(CacheKey::from_id(id, offset), block);
    }
}
