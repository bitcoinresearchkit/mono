// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

//! BRK's immutable byte slice.
//!
//! Small keys and values are stored inline. Larger values and their subslices
//! share one reference-counted allocation. Lengths are limited to 4 GiB.

#![deny(clippy::all, missing_docs, clippy::cargo)]
#![allow(
    clippy::cargo_common_metadata,
    reason = "not every internal workspace package is independently published"
)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::indexing_slicing)]
#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::needless_lifetimes
)]

mod builder;
mod byteview;

pub use byteview::ByteView;

#[doc(hidden)]
pub use byteview::Builder;
