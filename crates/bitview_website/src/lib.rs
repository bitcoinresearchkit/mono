//! Website router for Bitview: the embedded site, or in debug builds the repository's folder live.
//!
//! This crate provides website serving without any BRK data layer dependencies.
//!
//! See the `website` example for how to run a standalone server.

mod error;
mod handlers;
mod page_info;
mod router;
mod website;

pub(crate) use error::Error;
pub use router::router;
