//! HTTP cache layer. ETag-based revalidation with separate browser and CDN
//! directives (RFC 9213), plus serialized representations owned by the server:
//!
//! - [`CacheStrategy`] — *what kind of resource* the handler is returning
//!   (input enum picked by the route).
//! - [`CacheParams`]   — the *resolved* etag + Cache-Control + CDN-Cache-Control,
//!   derived from a strategy plus current chain tip.
//! - [`CdnCacheMode`]  — operator-level toggle for the CDN cached tier
//!   (process-global, set once via [`init`] from `Server::bind`).
//! - `TipJsonCache`    — serialized JSON reused while the exact chain tip is
//!   unchanged.

#[cfg(feature = "chain")]
mod blocks;
#[cfg(feature = "chain")]
mod mining;
mod mode;
mod params;
mod strategy;
#[cfg(any(feature = "chain", feature = "urpd"))]
mod tip_json;
#[cfg(feature = "urpd")]
mod urpd;

#[cfg(feature = "chain")]
pub(crate) use blocks::BlockCaches;
#[cfg(feature = "chain")]
pub(crate) use mining::MiningCaches;
pub use mode::CdnCacheMode;
pub use params::CacheParams;
pub(crate) use params::ErrorCachePolicy;
pub use strategy::CacheStrategy;
#[cfg(any(feature = "chain", feature = "urpd"))]
pub(crate) use tip_json::TipJsonCache;
#[cfg(feature = "urpd")]
pub(crate) use urpd::UrpdCaches;

pub use mode::init;
