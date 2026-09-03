use brk_types::{BlockHashPrefix, Version};

/// Cache strategy for HTTP responses.
///
/// The series strategy is computed directly in `api/series::serve` because
/// its parameters (total / end / hash) only become known after query
/// resolution, so it bypasses this enum and builds a
/// [`CacheParams`](super::CacheParams) via
/// [`CacheParams::series`](super::CacheParams::series).
pub enum CacheStrategy {
    /// Chain-dependent data (addresses, mining stats, txs, outspends).
    /// Etag = `t{tip_hash_prefix:x}`. Invalidates on any tip change including reorgs.
    Tip(BlockHashPrefix),

    /// Immutable data identified by hash in the URL (blocks by hash, confirmed tx data).
    /// Etag = `i{version}`. Permanent, only bumped when response format changes.
    Immutable(Version),

    /// Non-chain data tied to the deploy (validate-address, series catalog, pool list).
    /// Etag = `d{CARGO_PKG_VERSION}`. Invalidates on deploy.
    Deploy,

    /// Immutable data bound to a specific block (confirmed tx data, block status).
    /// Etag = `b{version}-{block_hash_prefix:x}`. Invalidates naturally on reorg.
    BlockBound(Version, BlockHashPrefix),

    /// Mutable state whose current representation is anchored to its latest
    /// relevant block (address state and latest pool-block pages).
    /// Etag = `a{version}-{block_hash_prefix:x}`. The CDN revalidates so later
    /// activity can replace the representation without waiting for a purge.
    ActivityBound(Version, BlockHashPrefix),

    /// Mutable data identified by a representation-specific hash.
    /// Etag = `l{hash:x}`. Uses the live CDN policy.
    LiveHash(u64),
}

impl CacheStrategy {
    #[inline]
    pub(crate) const fn tip_hash(&self) -> Option<BlockHashPrefix> {
        match self {
            Self::Tip(tip) => Some(*tip),
            _ => None,
        }
    }
}
