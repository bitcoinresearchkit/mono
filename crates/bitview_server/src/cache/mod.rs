//! HTTP cache layer. ETag-based revalidation with separate browser and CDN
//! directives (RFC 9213). Three concepts, one file each:
//!
//! - [`CacheStrategy`] — *what kind of resource* the handler is returning
//!   (input enum picked by the route).
//! - [`CacheParams`]   — the *resolved* etag + Cache-Control + CDN-Cache-Control,
//!   derived from a strategy plus current chain tip.
//! - [`CdnCacheMode`]  — operator-level toggle for the CDN cached tier
//!   (process-global, set once via [`init`] from `Server::bind`).

mod mode;
mod params;
mod strategy;

pub use mode::CdnCacheMode;
pub use params::CacheParams;
pub use strategy::CacheStrategy;

pub use mode::init;
