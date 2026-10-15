//! HTTP cache layer. ETag-based revalidation with separate browser and CDN
//! directives (RFC 9213), plus serialized representations owned by the server:
//!
//! - [`CacheParams`]   — etag + Cache-Control + CDN-Cache-Control, one
//!   constructor per kind of resource the handler is returning.
//! - [`CdnCacheMode`]  — operator-level toggle for the CDN cached tier
//!   (owned by each server instance).

mod mode;
mod params;
pub use mode::CdnCacheMode;
pub use params::CacheParams;
pub use params::ErrorCachePolicy;
