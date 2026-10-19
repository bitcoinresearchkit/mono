//! Website router for Bitview: embedded assets or a filesystem directory.
//!
//! This crate provides website serving without any BRK data layer dependencies.
//!
//! See the `website` example for how to run a standalone server.

mod error;
mod handlers;
mod router;
mod website;

pub(crate) use error::Error;
pub use router::router;
pub use website::Website;
