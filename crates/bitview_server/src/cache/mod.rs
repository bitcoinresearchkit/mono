//! HTTP cache policies: ETag revalidation with separate browser and CDN directives (RFC 9213).

mod mode;
mod params;
pub use mode::CdnCacheMode;
pub use params::{CDN_LIVE, CacheParams};
pub use params::ErrorCachePolicy;
