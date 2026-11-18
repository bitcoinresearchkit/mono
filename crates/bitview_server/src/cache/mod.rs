//! HTTP cache policies: ETag revalidation with separate browser and CDN directives (RFC 9213).

mod mode;
pub use mode::CdnCacheMode;
