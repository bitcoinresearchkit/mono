//! Standalone website serving for BRK.
//!
//! This crate provides website serving without any BRK data layer dependencies.
//! It can serve the embedded website or from a filesystem path.
//!
//! See the `website` example for how to run a standalone server.

mod error;
mod handlers;
mod headers;
mod router;
mod website;

pub use error::Error;
pub use headers::HeaderMapExtended;
pub use router::router;
pub use website::{EMBEDDED_WEBSITE, Website};
