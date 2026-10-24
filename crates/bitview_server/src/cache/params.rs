use axum::http::HeaderMap;
use bitview_primitives::BlockHashPrefix;
#[cfg(feature = "chain")]
use bitview_query::RepresentationId;
#[cfg(feature = "series")]
use bitview_query::{ResolvedQuery, SeriesAnchor};
use brk_types::Version;

use super::mode::CdnCacheMode;
use crate::{etag::Etag, extended::HeaderMapExtended};

// Browser-facing: always revalidate via ETag. `no-cache` means "cache it but
// check before use" (not "don't cache"); ETag makes the check cheap.
const CC: &str = "public, no-cache, must-revalidate";
/// Reused for a second, then asked again: what a CDN may do with a live answer.
pub const CDN_LIVE: &str = "public, max-age=1, must-revalidate";
const CDN_STABLE: &str = "public, max-age=31536000, immutable";

// Revalidating errors are briefly cacheable, but never served stale: even a permanent input
// error's body and verdict can change with a release, and browsers are never purged. Transient
// and private errors are never stored. Browser and CDN policies intentionally match for errors.
const CC_REVALIDATE: &str = "public, max-age=1, must-revalidate";
const CC_ERROR_NO_STORE: &str = "no-store";

#[derive(Clone, Copy)]
pub enum ErrorCachePolicy {
    Revalidate,
    NoStore,
}

impl ErrorCachePolicy {
    fn cache_control(self) -> &'static str {
        match self {
            Self::Revalidate => CC_REVALIDATE,
            Self::NoStore => CC_ERROR_NO_STORE,
        }
    }
}

/// Resolved cache parameters: an ETag plus the two Cache-Control directives.
#[derive(Clone)]
pub struct CacheParams {
    pub etag: Etag,
    cache_control: &'static str,
    cdn_cache_control: &'static str,
}

impl CacheParams {
    /// The stable CDN tier when `settled`: bytes only a release or a reorg below the reorg window
    /// can change. With [`CdnCacheMode::Aggressive`], held up to a year until the release's purge.
    pub fn stable_if(mut self, settled: bool, cdn_cache_mode: CdnCacheMode) -> Self {
        if settled && cdn_cache_mode == CdnCacheMode::Aggressive {
            self.cdn_cache_control = CDN_STABLE;
        }
        self
    }

    /// Live response with an exact, versioned representation tag: browsers revalidate, the CDN
    /// reuses it for a second without stale reuse.
    pub fn live(tag: impl Into<Etag>) -> Self {
        Self {
            etag: tag.into(),
            cache_control: CC,
            cdn_cache_control: CDN_LIVE,
        }
    }

    /// Mutable data identified by a representation-specific hash: `l{hash:x}`.
    pub fn live_hash(hash: u64) -> Self {
        Self::live(format!("l{hash:x}"))
    }

    /// Data fixed for the release (the empty search): `i{version}`, bumped when the response
    /// format changes.
    pub fn immutable(version: Version, cdn_cache_mode: CdnCacheMode) -> Self {
        Self::live(format!("i{version}")).stable_if(true, cdn_cache_mode)
    }

    /// Mutable state whose current representation is anchored to its latest relevant block
    /// (address state, UTXO set points): `a{version}-{block_hash_prefix:x}`. The CDN
    /// revalidates so later activity can replace it without waiting for a purge; a point its
    /// URL pins below the reorg window is [`Self::stable_if`] settled.
    pub fn activity_bound(version: Version, prefix: BlockHashPrefix) -> Self {
        Self::live(format!("a{version}-{:x}", *prefix))
    }

    /// Release-bound response (validate-address, version): `d`, after the release
    /// prefix every tag starts with.
    pub fn deploy() -> Self {
        Self::revalidate("d".to_string())
    }

    /// A response a second's reuse cannot mislead (block by hash, tip, catalogs, deploy-tied):
    /// browsers and the CDN reuse it for a second, then revalidate, without stale reuse.
    pub fn revalidate(tag: impl Into<Etag>) -> Self {
        Self {
            etag: tag.into(),
            cache_control: CC_REVALIDATE,
            cdn_cache_control: CC_REVALIDATE,
        }
    }

    #[cfg(feature = "chain")]
    pub fn representation(version: Version, identity: RepresentationId) -> Self {
        match identity {
            RepresentationId::Content(hash) => Self::live_hash(hash),
            // Anchor identity and availability are distinct: a displaced
            // confirmed transaction must be resolved again even at depth.
            RepresentationId::Block(hash) => Self::live(format!("tx3-{version}-{hash}")),
        }
    }

    /// Series query: the resolved [`SeriesAnchor`] picks the stable or the live CDN tier.
    ///
    /// Etag shapes:
    /// - historical: `s{v}-r{start}-{end}-{anchor:x}`, the anchor being the last block the rows
    ///   depend on (for calendar indexes, the following period's first block): stable across
    ///   appends, changed by any reorg replacing those rows.
    /// - tip: `s{v}-t{tip_hash:x}`. Invalidates per block, reorg-safe.
    #[cfg(feature = "series")]
    pub fn series(query: &ResolvedQuery, cdn_cache_mode: CdnCacheMode) -> Self {
        let v = u32::from(query.version());
        let range = query.range();
        match query.anchor() {
            SeriesAnchor::Historical(hash) => {
                Self::live(format!("s{v}-r{}-{}-{:x}", range.start, range.end, *hash))
                    .stable_if(true, cdn_cache_mode)
            }
            SeriesAnchor::Tip(hash) => Self::live(format!("s{v}-t{:x}", *hash)),
        }
    }

    /// Apply an error cache policy. Error responses deliberately have no ETag:
    /// a conditional error request must receive the error status again, not 304.
    pub fn apply_error_cache_control(headers: &mut HeaderMap, policy: ErrorCachePolicy) {
        let cache_control = policy.cache_control();
        headers.insert_cache_control(cache_control);
        headers.insert_cdn_cache_control(cache_control);
    }

    /// Write this cache policy (etag + cache-control + cdn-cache-control) onto a response's headers.
    pub fn apply_to(&self, headers: &mut HeaderMap) {
        self.etag.insert(headers);
        headers.insert_cache_control(self.cache_control);
        headers.insert_cdn_cache_control(self.cdn_cache_control);
    }
}
