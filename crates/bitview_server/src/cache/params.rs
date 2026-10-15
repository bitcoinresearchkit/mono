use axum::http::HeaderMap;
use bitview_primitives::BlockHashPrefix;
#[cfg(feature = "chain")]
use bitview_query::RepresentationId;
#[cfg(feature = "series")]
use bitview_query::SeriesAnchor;
use brk_types::Version;

use super::mode::CdnCacheMode;
use crate::{VERSION, etag::Etag, extended::HeaderMapExtended};

// Browser-facing: always revalidate via ETag. `no-cache` means "cache it but
// check before use" (not "don't cache"); ETag makes the check cheap.
const CC: &str = "public, no-cache, must-revalidate";
const CDN_LIVE: &str = "public, max-age=1, must-revalidate";

// Revalidating errors are briefly cacheable, but never served stale. Permanent
// input errors can be cached indefinitely. Transient and private errors are
// never stored. Browser and CDN policies intentionally match for errors.
const CC_REVALIDATE: &str = "public, max-age=1, must-revalidate";
const CC_ERROR_IMMUTABLE: &str = "public, max-age=31536000, immutable";
const CC_ERROR_NO_STORE: &str = "no-store";

#[derive(Clone, Copy)]
pub enum ErrorCachePolicy {
    Revalidate,
    Immutable,
    NoStore,
}

impl ErrorCachePolicy {
    fn cache_control(self) -> &'static str {
        match self {
            Self::Revalidate => CC_REVALIDATE,
            Self::Immutable => CC_ERROR_IMMUTABLE,
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
    const fn cdn_cache_control(mode: CdnCacheMode) -> &'static str {
        match mode {
            CdnCacheMode::Live => CDN_LIVE,
            CdnCacheMode::Aggressive => "public, max-age=31536000, immutable",
        }
    }

    /// Live response with an exact, versioned representation tag.
    pub fn live(tag: String) -> Self {
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

    /// Immutable data identified by hash in the URL (blocks by hash, confirmed tx data):
    /// `i{version}`. Permanent, only bumped when the response format changes.
    pub fn immutable(version: Version, cdn_cache_mode: CdnCacheMode) -> Self {
        Self {
            etag: format!("i{version}").into(),
            cache_control: CC,
            cdn_cache_control: Self::cdn_cache_control(cdn_cache_mode),
        }
    }

    /// Mutable state whose current representation is anchored to its latest relevant block
    /// (address state, UTXO set points): `a{version}-{block_hash_prefix:x}`. The CDN
    /// revalidates so later activity can replace it without waiting for a purge.
    pub fn activity_bound(version: Version, prefix: BlockHashPrefix) -> Self {
        Self::live(format!("a{version}-{:x}", *prefix))
    }

    /// Deploy-tied response (validate-address, OpenAPI spec, scalar bundle): `d{VERSION}`.
    /// Browser and CDN freshness are capped at one second without stale reuse.
    pub fn deploy() -> Self {
        Self::revalidate(format!("d{VERSION}").into())
    }

    /// Short freshness for a mutable URL with an already-resolved validator.
    pub fn revalidate(etag: Etag) -> Self {
        Self {
            etag,
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

    /// Series query: historical rows get CACHED, rows reaching the reorg window get LIVE.
    ///
    /// Etag shapes (see [`SeriesAnchor`]):
    /// - historical: `s{v}-r{start}-{end}-{anchor:x}`, the anchor being the last block the rows
    ///   cover: stable across appends, changed by any reorg replacing those rows.
    /// - tip: `s{v}-t{tip_hash:x}`. Invalidates per block, reorg-safe.
    #[cfg(feature = "series")]
    pub fn series(
        version: Version,
        start: usize,
        end: usize,
        anchor: SeriesAnchor,
        cdn_cache_mode: CdnCacheMode,
    ) -> Self {
        let v = u32::from(version);
        match anchor {
            SeriesAnchor::Historical(hash) => Self {
                etag: format!("s{v}-r{start}-{end}-{:x}", *hash).into(),
                cache_control: CC,
                cdn_cache_control: Self::cdn_cache_control(cdn_cache_mode),
            },
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

    pub fn matches_etag(&self, headers: &HeaderMap) -> bool {
        self.etag.matches(headers)
    }

    /// Write this cache policy (etag + cache-control + cdn-cache-control) onto a response's headers.
    pub fn apply_to(&self, headers: &mut HeaderMap) {
        self.etag.insert(headers);
        headers.insert_cache_control(self.cache_control);
        headers.insert_cdn_cache_control(self.cdn_cache_control);
    }
}
