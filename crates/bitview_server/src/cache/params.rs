use axum::http::HeaderMap;
use brk_types::{BlockHashPrefix, Version};

use crate::{VERSION, etag::Etag, extended::HeaderMapExtended};

use super::{mode::CdnCacheMode, strategy::CacheStrategy};

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

    fn immutable(version: Version, cdn_cache_mode: CdnCacheMode) -> Self {
        Self {
            etag: format!("i{version}").into(),
            cache_control: CC,
            cdn_cache_control: Self::cdn_cache_control(cdn_cache_mode),
        }
    }

    fn activity_bound(version: Version, prefix: BlockHashPrefix) -> Self {
        Self {
            etag: format!("a{version}-{:x}", *prefix).into(),
            cache_control: CC,
            cdn_cache_control: CDN_LIVE,
        }
    }

    /// Deploy-tied response: etag from the build version. Used directly
    /// by static handlers (OpenAPI spec, scalar bundle) that don't have
    /// a [`CacheStrategy`] context.
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

    fn live_hash(hash: u64) -> Self {
        Self {
            etag: format!("l{hash:x}").into(),
            cache_control: CC,
            cdn_cache_control: CDN_LIVE,
        }
    }

    /// Series query: tail-bound gets LIVE, historical gets CACHED.
    ///
    /// `stable_count` is the count of leading entries provably immutable across
    /// a 6-block reorg (per `Index::cache_class()` + `Query::stable_count`).
    /// `None` (any series with mutable existing entries) forces the tail branch
    /// for every range.
    ///
    /// Etag shapes:
    /// - historical (`end <= stable_count`): `s{v}-h{start}-{end}`. Pure
    ///   range, stable across appends and reorgs of the volatile tail.
    /// - tail (`end > stable_count` or `stable_count.is_none()`):
    ///   `s{v}-t{tip_hash:x}`. Invalidates per-block, reorg-safe.
    ///
    /// The `h`/`t` discriminator after `s{v}-` prevents collision with old
    /// `s{v}-{number}` ETags from before the migration.
    pub fn series(
        version: Version,
        start: usize,
        end: usize,
        stable_count: Option<usize>,
        hash: BlockHashPrefix,
        cdn_cache_mode: CdnCacheMode,
    ) -> Self {
        let v = u32::from(version);
        match stable_count {
            Some(s) if end <= s => Self {
                etag: format!("s{v}-h{start}-{end}").into(),
                cache_control: CC,
                cdn_cache_control: Self::cdn_cache_control(cdn_cache_mode),
            },
            _ => Self {
                etag: format!("s{v}-t{:x}", *hash).into(),
                cache_control: CC,
                cdn_cache_control: CDN_LIVE,
            },
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

    pub fn resolve(strategy: &CacheStrategy, cdn_cache_mode: CdnCacheMode) -> Self {
        match strategy {
            CacheStrategy::Live(etag) => Self {
                etag: etag.clone(),
                cache_control: CC,
                cdn_cache_control: CDN_LIVE,
            },

            CacheStrategy::Immutable(v) => Self::immutable(*v, cdn_cache_mode),

            CacheStrategy::ActivityBound(v, prefix) => Self::activity_bound(*v, *prefix),
            CacheStrategy::Deploy => Self::deploy(),
            CacheStrategy::LiveHash(hash) => Self::live_hash(*hash),
        }
    }
}
