use crate::config::{
    BloomConstructionPolicy, FilterPolicy, FilterPolicyEntry, PartitioningPolicy, PinningPolicy,
    RestartIntervalPolicy,
};
use lsm_tree::{
    CompressionType,
    config::{BlockSizePolicy, CompressionPolicy, HashRatioPolicy},
};
use std::path::Path;

/// Immutable-table configuration for a keyspace.
pub struct CreateOptions {
    data_block_restart_interval_policy: RestartIntervalPolicy,
    index_block_pinning_policy: PinningPolicy,
    filter_block_pinning_policy: PinningPolicy,
    filter_block_partitioning_policy: PartitioningPolicy,
    index_block_partitioning_policy: PartitioningPolicy,
    expect_point_read_hits: bool,
    filter_policy: FilterPolicy,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            data_block_restart_interval_policy: RestartIntervalPolicy::new([10, 16]),
            index_block_pinning_policy: PinningPolicy::new([true, true, false]),
            filter_block_pinning_policy: PinningPolicy::new([true, false]),
            index_block_partitioning_policy: PartitioningPolicy::new([false, false, false, true]),
            filter_block_partitioning_policy: PartitioningPolicy::new([false, false, false, true]),
            expect_point_read_hits: false,
            filter_policy: FilterPolicy::new([
                FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)),
                FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),
            ]),
        }
    }
}

impl CreateOptions {
    /// Sets the restart interval inside data blocks.
    #[must_use]
    pub fn data_block_restart_interval_policy(mut self, policy: RestartIntervalPolicy) -> Self {
        self.data_block_restart_interval_policy = policy;
        self
    }

    /// Sets the filter-block pinning policy.
    #[must_use]
    pub fn filter_block_pinning_policy(mut self, policy: PinningPolicy) -> Self {
        self.filter_block_pinning_policy = policy;
        self
    }

    /// Sets the index-block pinning policy.
    #[must_use]
    pub fn index_block_pinning_policy(mut self, policy: PinningPolicy) -> Self {
        self.index_block_pinning_policy = policy;
        self
    }

    /// Sets the filter-block partitioning policy.
    #[must_use]
    pub fn filter_block_partitioning_policy(mut self, policy: PartitioningPolicy) -> Self {
        self.filter_block_partitioning_policy = policy;
        self
    }

    /// Sets the index-block partitioning policy.
    #[must_use]
    pub fn index_block_partitioning_policy(mut self, policy: PartitioningPolicy) -> Self {
        self.index_block_partitioning_policy = policy;
        self
    }

    /// Omits last-level filters when point reads are normally successful.
    #[must_use]
    pub fn expect_point_read_hits(mut self, enabled: bool) -> Self {
        self.expect_point_read_hits = enabled;
        self
    }

    /// Sets the Bloom-filter policy.
    #[must_use]
    pub fn filter_policy(mut self, policy: FilterPolicy) -> Self {
        self.filter_policy = policy;
        self
    }

    /// Builds the underlying LSM-tree configuration.
    #[doc(hidden)]
    #[must_use]
    pub fn tree_config(self, path: &Path, database: &crate::db_config::Config) -> lsm_tree::Config {
        let config = lsm_tree::Config::new(path)
            .use_cache(database.cache.clone())
            .use_descriptor_table(Some(database.descriptor_table.clone()))
            .data_block_size_policy(BlockSizePolicy::all(4 * 1_024))
            .data_block_hash_ratio_policy(HashRatioPolicy::all(0.0))
            .data_block_restart_interval_policy(self.data_block_restart_interval_policy)
            .index_block_pinning_policy(self.index_block_pinning_policy)
            .filter_block_pinning_policy(self.filter_block_pinning_policy)
            .index_block_partitioning_policy(self.index_block_partitioning_policy)
            .filter_block_partitioning_policy(self.filter_block_partitioning_policy)
            .expect_point_read_hits(self.expect_point_read_hits)
            .filter_policy(self.filter_policy)
            .index_block_compression_policy(CompressionPolicy::all(CompressionType::None));

        config.data_block_compression_policy(CompressionPolicy::new([
            CompressionType::None,
            CompressionType::None,
            CompressionType::Lz4,
        ]))
    }
}
