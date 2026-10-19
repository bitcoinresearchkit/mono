/// CDN caching strategy for stable responses (immutable / block-bound /
/// historical series). Live-tier and deployment-bound responses are unaffected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CdnCacheMode {
    /// One second of CDN freshness, then revalidation. No stale-on-error reuse.
    #[default]
    Live,
    /// CDN holds stable responses for up to a year as `immutable` (RFC 8246). Operator must
    /// purge on every deploy.
    Aggressive,
}
