/// What the CDN is told about settled answers (those a URL pins below the reorg window). Live
/// answers get one second of freshness either way.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CdnCacheMode {
    /// One second of CDN freshness, then revalidation. No stale-on-error reuse.
    #[default]
    Live,
    /// The CDN holds settled answers for up to a year as `immutable` (RFC 8246). The operator
    /// must purge it on every release, and on a reorg deeper than the reorg window.
    Aggressive,
}
