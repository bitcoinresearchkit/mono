use bitview_primitives::BlockHashPrefix;
use brk_types::Version;

use crate::etag::Etag;

/// Cache strategy for HTTP responses.
///
/// The series strategy is computed directly in `api/series::serve` because
/// its parameters (total / end / hash) only become known after query
/// resolution, so it bypasses this enum and builds a
/// [`CacheParams`](super::CacheParams) via
/// [`CacheParams::series`](super::CacheParams::series).
pub enum CacheStrategy {
    /// Live response with an exact, versioned representation tag.
    Live(Etag),

    /// Immutable data identified by hash in the URL (blocks by hash, confirmed tx data).
    /// Etag = `i{version}`. Permanent, only bumped when response format changes.
    Immutable(Version),

    /// Non-chain data tied to the deploy (validate-address, series catalog, pool list).
    /// Etag = `d{CARGO_PKG_VERSION}`. Changes with the package version.
    /// Browser and CDN freshness are capped at one second without stale reuse.
    Deploy,

    /// Mutable state whose current representation is anchored to its latest
    /// relevant block (address state and latest pool-block pages).
    /// Etag = `a{version}-{block_hash_prefix:x}`. The CDN revalidates so later
    /// activity can replace the representation without waiting for a purge.
    ActivityBound(Version, BlockHashPrefix),

    /// Mutable data identified by a representation-specific hash.
    /// Etag = `l{hash:x}`. Uses the live CDN policy.
    LiveHash(u64),
}
