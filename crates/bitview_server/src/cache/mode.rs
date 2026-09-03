use std::sync::OnceLock;

// CDN-facing (RFC 9213). Two tiers: live (chain-state, changes per block /
// mempool event) and cached (stable, ETag-invalidated).
//
// `Live` is always-revalidate: origin handles every request, cheap via ETag,
// no risk of stale data for self-hosters who don't run a purge step.
// `Aggressive` caches stable responses for up to a year and treats them as
// `immutable` (RFC 8246) — the operator must purge the CDN on every deploy.
pub const CDN_LIVE: &str = "public, max-age=1, stale-if-error=300";
const CDN_AGGRESSIVE: &str = "public, max-age=31536000, immutable";

/// CDN caching strategy for stable responses (immutable / deploy / block-bound /
/// historical series). Live-tier responses (`Tip`, `LiveHash`, tail series)
/// are unaffected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CdnCacheMode {
    /// Origin revalidates every response via ETag. No CDN purge required. Safe default.
    #[default]
    Live,
    /// CDN holds stable responses for up to a year and treats them as immutable.
    /// Operator must purge on every deploy.
    Aggressive,
}

impl CdnCacheMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Live => CDN_LIVE,
            Self::Aggressive => CDN_AGGRESSIVE,
        }
    }
}

static CDN_CACHE_MODE: OnceLock<CdnCacheMode> = OnceLock::new();

/// Set once at server startup. Subsequent calls are ignored (first-wins). If a
/// later call conflicts with the existing mode, log a warning so the mismatch
/// is visible in plugin / orchestrator setups that spin up multiple servers in
/// the same process.
pub fn init(mode: CdnCacheMode) {
    if CDN_CACHE_MODE.set(mode).is_err() {
        let existing = CDN_CACHE_MODE.get().copied().unwrap_or_default();
        if existing != mode {
            tracing::warn!(
                "cache::init called with {mode:?} but mode is already set to {existing:?}; ignoring"
            );
        }
    }
}

/// Cached-tier directive for stable responses. Defaults to `Live` if [`init`]
/// was never called (tests, library use without a `Server`).
pub fn cdn_cached() -> &'static str {
    CDN_CACHE_MODE.get().copied().unwrap_or_default().as_str()
}
