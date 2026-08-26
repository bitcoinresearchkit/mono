use axum::http::HeaderMap;
use brk_types::{BlockHashPrefix, Version};

use crate::{VERSION, etag::Etag, extended::HeaderMapExtended};

use super::{
    mode::{CDN_LIVE, cdn_cached},
    strategy::CacheStrategy,
};

// Browser-facing: always revalidate via ETag. `no-cache` means "cache it but
// check before use" (not "don't cache"); ETag makes the check cheap.
const CC: &str = "public, no-cache, stale-if-error=86400";

// Errors: short, must-revalidate, no `stale-if-error` (we don't want a 24h-old
// error served when origin recovers). Same string for browser and CDN.
const CC_ERROR: &str = "public, max-age=1, must-revalidate";

/// Resolved cache parameters: an ETag plus the two Cache-Control directives.
pub struct CacheParams {
    pub etag: Etag,
    cache_control: &'static str,
    cdn_cache_control: &'static str,
}

impl CacheParams {
    fn tip(tip: BlockHashPrefix) -> Self {
        Self {
            etag: format!("t{:x}", *tip).into(),
            cache_control: CC,
            cdn_cache_control: CDN_LIVE,
        }
    }

    fn immutable(version: Version) -> Self {
        Self {
            etag: format!("i{version}").into(),
            cache_control: CC,
            cdn_cache_control: cdn_cached(),
        }
    }

    fn block_bound(version: Version, prefix: BlockHashPrefix) -> Self {
        Self {
            etag: format!("b{version}-{:x}", *prefix).into(),
            cache_control: CC,
            cdn_cache_control: cdn_cached(),
        }
    }

    /// Deploy-tied response: etag from the build version. Used directly
    /// by static handlers (OpenAPI spec, scalar bundle) that don't have
    /// a [`CacheStrategy`] context.
    pub fn deploy() -> Self {
        Self {
            etag: format!("d{VERSION}").into(),
            cache_control: CC,
            cdn_cache_control: cdn_cached(),
        }
    }

    fn mempool_hash(hash: u64) -> Self {
        Self {
            etag: format!("m{hash:x}").into(),
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
    ) -> Self {
        let v = u32::from(version);
        match stable_count {
            Some(s) if end <= s => Self {
                etag: format!("s{v}-h{start}-{end}").into(),
                cache_control: CC,
                cdn_cache_control: cdn_cached(),
            },
            _ => Self {
                etag: format!("s{v}-t{:x}", *hash).into(),
                cache_control: CC,
                cdn_cache_control: CDN_LIVE,
            },
        }
    }

    /// Error response: keeps the originating ETag (so retries can 304),
    /// uses [`CC_ERROR`] for both browser and CDN.
    pub fn error(etag: Etag) -> Self {
        Self {
            etag,
            cache_control: CC_ERROR,
            cdn_cache_control: CC_ERROR,
        }
    }

    /// Apply error cache-control headers without an ETag. Used for synthesized
    /// errors (panics, fallback handlers) that have no resource etag.
    pub fn apply_error_cache_control(headers: &mut HeaderMap) {
        headers.insert_cache_control(CC_ERROR);
        headers.insert_cdn_cache_control(CC_ERROR);
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

    pub fn resolve(strategy: &CacheStrategy, tip: BlockHashPrefix) -> Self {
        match strategy {
            CacheStrategy::Tip => Self::tip(tip),
            CacheStrategy::Immutable(v) => Self::immutable(*v),
            CacheStrategy::BlockBound(v, prefix) => Self::block_bound(*v, *prefix),
            CacheStrategy::Deploy => Self::deploy(),
            CacheStrategy::MempoolHash(hash) => Self::mempool_hash(*hash),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(n: u32) -> Version {
        Version::new(n)
    }

    fn h(n: u64) -> BlockHashPrefix {
        BlockHashPrefix::from(n)
    }

    #[test]
    fn series_tail_when_end_exceeds_stable_count() {
        let p = CacheParams::series(v(3), 0, 60, Some(50), h(0xabcd));
        assert_eq!(p.etag.as_str(), "s3-tabcd");
    }

    #[test]
    fn series_historical_when_end_at_or_below_stable_count() {
        let p = CacheParams::series(v(3), 10, 50, Some(50), h(0xabcd));
        assert_eq!(p.etag.as_str(), "s3-h10-50");
    }

    #[test]
    fn series_historical_ignores_tip_hash() {
        let a = CacheParams::series(v(3), 0, 50, Some(100), h(0xabcd));
        let b = CacheParams::series(v(3), 0, 50, Some(100), h(0xdead));
        assert_eq!(a.etag.as_str(), b.etag.as_str());
    }

    #[test]
    fn series_tail_changes_with_tip_hash() {
        let a = CacheParams::series(v(3), 0, 100, Some(50), h(0xabcd));
        let b = CacheParams::series(v(3), 0, 100, Some(50), h(0xdead));
        assert_ne!(a.etag.as_str(), b.etag.as_str());
    }

    #[test]
    fn series_mutable_class_always_tail() {
        let small = CacheParams::series(v(3), 0, 5, None, h(0xabcd));
        let large = CacheParams::series(v(3), 0, 1_000_000, None, h(0xabcd));
        assert_eq!(small.etag.as_str(), "s3-tabcd");
        assert_eq!(large.etag.as_str(), "s3-tabcd");
    }

    #[test]
    fn series_at_stable_boundary_is_historical() {
        let p = CacheParams::series(v(3), 0, 50, Some(50), h(0xabcd));
        assert_eq!(p.etag.as_str(), "s3-h0-50");
    }

    #[test]
    fn series_just_past_stable_boundary_is_tail() {
        let p = CacheParams::series(v(3), 0, 51, Some(50), h(0xabcd));
        assert_eq!(p.etag.as_str(), "s3-tabcd");
    }

    #[test]
    fn series_different_ranges_get_different_etags() {
        let a = CacheParams::series(v(3), 0, 50, Some(100), h(0xabcd));
        let b = CacheParams::series(v(3), 10, 50, Some(100), h(0xabcd));
        assert_ne!(a.etag.as_str(), b.etag.as_str());
    }
}
