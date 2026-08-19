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
    Tip,

    /// Immutable data identified by hash in the URL (blocks by hash, confirmed tx data).
    /// Etag = `i{version}`. Permanent, only bumped when response format changes.
    Immutable(Version),

    /// Non-chain data tied to the deploy (validate-address, series catalog, pool list).
    /// Etag = `d{CARGO_PKG_VERSION}`. Invalidates on deploy.
    Deploy,

    /// Immutable data bound to a specific block (confirmed tx data, block status).
    /// Etag = `b{version}-{block_hash_prefix:x}`. Invalidates naturally on reorg.
    BlockBound(Version, BlockHashPrefix),

    /// Mempool data, etag from next projected block hash.
    /// Etag = `m{hash:x}`. Invalidates on mempool change.
    MempoolHash(u64),
}
