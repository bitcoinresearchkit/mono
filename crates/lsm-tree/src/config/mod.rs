// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Cache, CompressionType, DescriptorTable, Result, Tree, path::absolute_path};

mod block_size;
mod compression;
mod filter;
mod partitioning;
mod pinning;
mod restart_interval;

#[cfg(test)]
#[path = "../../tests/unit/config.rs"]
mod tests;

pub use block_size::BlockSizePolicy;
pub use compression::CompressionPolicy;
pub use filter::{BloomConstructionPolicy, FilterPolicy, FilterPolicyEntry};
pub use partitioning::PartitioningPolicy;
pub use pinning::PinningPolicy;
pub use restart_interval::RestartIntervalPolicy;

/// Tree configuration builder
pub struct Config {
    /// Folder path
    #[doc(hidden)]
    pub(crate) path: PathBuf,

    /// Block cache to use
    #[doc(hidden)]
    pub(crate) cache: Arc<Cache>,

    /// Descriptor table to use
    #[doc(hidden)]
    pub(crate) descriptor_table: Option<Arc<DescriptorTable>>,

    /// What type of compression is used for data blocks
    pub(crate) data_block_compression_policy: CompressionPolicy,

    /// What type of compression is used for index blocks
    pub(crate) index_block_compression_policy: CompressionPolicy,

    /// Restart interval inside data blocks
    pub(crate) data_block_restart_interval_policy: RestartIntervalPolicy,

    /// Block size of data blocks
    pub(crate) data_block_size_policy: BlockSizePolicy,

    /// Whether to pin index blocks
    pub(crate) index_block_pinning_policy: PinningPolicy,

    /// Whether to pin filter blocks
    pub(crate) filter_block_pinning_policy: PinningPolicy,

    /// Whether to partition index blocks
    pub(crate) index_block_partitioning_policy: PartitioningPolicy,

    /// Whether to partition filter blocks
    pub(crate) filter_block_partitioning_policy: PartitioningPolicy,

    /// If `true`, the last level will not build filters, reducing the filter size of a database
    /// by ~90% typically
    pub(crate) expect_point_read_hits: bool,

    /// Filter construction policy
    pub(crate) filter_policy: FilterPolicy,
}

impl Config {
    /// Initializes a tree configuration rooted at `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: absolute_path(path.as_ref()),
            descriptor_table: Some(Arc::new(DescriptorTable::new(256))),
            cache: Arc::new(Cache::with_capacity_bytes(16 * 1_024 * 1_024)),
            data_block_restart_interval_policy: RestartIntervalPolicy::all(16),
            data_block_size_policy: BlockSizePolicy::all(4_096),
            index_block_pinning_policy: PinningPolicy::new([true, true, false]),
            filter_block_pinning_policy: PinningPolicy::new([true, false]),
            index_block_partitioning_policy: PartitioningPolicy::new([false, false, false, true]),
            filter_block_partitioning_policy: PartitioningPolicy::new([false, false, false, true]),
            data_block_compression_policy: CompressionPolicy::new([
                CompressionType::None,
                CompressionType::Lz4,
            ]),
            index_block_compression_policy: CompressionPolicy::all(CompressionType::None),
            filter_policy: FilterPolicy::all(FilterPolicyEntry::Bloom(
                BloomConstructionPolicy::BitsPerKey(10.0),
            )),
            expect_point_read_hits: false,
        }
    }

    /// Sets the global cache.
    ///
    /// You can create a global [`Cache`] and share it between multiple
    /// trees to cap global cache memory usage.
    ///
    /// Defaults to a cache with 16 MiB of capacity *per tree*.
    #[must_use]
    pub fn use_cache(mut self, cache: Arc<Cache>) -> Self {
        self.cache = cache;
        self
    }

    /// Sets the file descriptor cache.
    ///
    /// Can be shared across trees.
    #[must_use]
    pub fn use_descriptor_table(mut self, descriptor_table: Option<Arc<DescriptorTable>>) -> Self {
        self.descriptor_table = descriptor_table;
        self
    }

    /// If `true`, the last level will not build filters, reducing the filter size of a database
    /// by ~90% typically.
    ///
    /// **Enable this only if you know that point reads generally are expected to find a key-value pair.**
    #[must_use]
    pub fn expect_point_read_hits(mut self, b: bool) -> Self {
        self.expect_point_read_hits = b;
        self
    }

    /// Sets the partitioning policy for filter blocks.
    #[must_use]
    pub fn filter_block_partitioning_policy(mut self, policy: PartitioningPolicy) -> Self {
        self.filter_block_partitioning_policy = policy;
        self
    }

    /// Sets the partitioning policy for index blocks.
    #[must_use]
    pub fn index_block_partitioning_policy(mut self, policy: PartitioningPolicy) -> Self {
        self.index_block_partitioning_policy = policy;
        self
    }

    /// Sets the pinning policy for filter blocks.
    #[must_use]
    pub fn filter_block_pinning_policy(mut self, policy: PinningPolicy) -> Self {
        self.filter_block_pinning_policy = policy;
        self
    }

    /// Sets the pinning policy for index blocks.
    #[must_use]
    pub fn index_block_pinning_policy(mut self, policy: PinningPolicy) -> Self {
        self.index_block_pinning_policy = policy;
        self
    }

    /// Sets the restart interval inside data blocks.
    ///
    /// A higher restart interval saves space while increasing lookup times
    /// inside data blocks.
    ///
    /// Default = 16
    #[must_use]
    pub fn data_block_restart_interval_policy(mut self, policy: RestartIntervalPolicy) -> Self {
        self.data_block_restart_interval_policy = policy;
        self
    }

    /// Sets the filter construction policy.
    #[must_use]
    pub fn filter_policy(mut self, policy: FilterPolicy) -> Self {
        self.filter_policy = policy;
        self
    }

    /// Sets the compression method for data blocks.
    #[must_use]
    pub fn data_block_compression_policy(mut self, policy: CompressionPolicy) -> Self {
        self.data_block_compression_policy = policy;
        self
    }

    /// Sets the compression method for index blocks.
    #[must_use]
    pub fn index_block_compression_policy(mut self, policy: CompressionPolicy) -> Self {
        self.index_block_compression_policy = policy;
        self
    }

    /// Sets the data block size policy.
    #[must_use]
    pub fn data_block_size_policy(mut self, policy: BlockSizePolicy) -> Self {
        self.data_block_size_policy = policy;
        self
    }

    /// Opens a tree using the config.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn open(self) -> Result<Tree> {
        Tree::open(self)
    }
}
