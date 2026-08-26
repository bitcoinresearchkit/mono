// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

//! BRK's table-only log-structured merge tree.
//!
//! Strictly sorted batches are written directly to immutable tables and
//! atomically published as a new version. Reads use the latest published table
//! layout, while leveled compaction bounds read amplification and disk usage.
//!
//! Keys are limited to 65536 bytes, values are limited to 2^32 bytes. As is normal with any kind of storage
//! engine, larger keys and values have a bigger performance impact.

#![doc(html_logo_url = "https://raw.githubusercontent.com/fjall-rs/lsm-tree/main/logo.png")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/fjall-rs/lsm-tree/main/logo.png")]
#![deny(clippy::all, missing_docs, clippy::cargo)]
#![allow(
    clippy::cargo_common_metadata,
    reason = "not every internal workspace package is independently published"
)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::indexing_slicing)]
#![warn(clippy::pedantic, clippy::nursery)]
#![warn(clippy::expect_used)]
#![allow(clippy::missing_const_for_fn)]
#![warn(clippy::multiple_crate_versions)]
#![allow(clippy::option_if_let_else)]
#![warn(clippy::redundant_feature_names)]
#![cfg_attr(
    test,
    allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::indexing_slicing,
        clippy::items_after_statements,
        clippy::too_many_lines,
        clippy::unwrap_used,
        clippy::useless_vec,
        reason = "test fixtures favor direct assertions and intentionally bounded inputs"
    )
)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

macro_rules! fail_iter {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return Some(Err(e.into())),
        }
    };
}

macro_rules! unwrap {
    ($x:expr) => {{ $x.expect("should read") }};
}

mod boxed_iterator;
#[doc(hidden)]
mod cache;

mod checksum;
mod coding;

mod compaction;
mod compression;

/// Configuration
pub mod config;

mod descriptor_table;
mod file_accessor;

mod double_ended_peekable;
mod error;

mod file;

mod hash;
mod key;
mod key_range;
mod run_reader;
mod run_scanner;

mod merge;
mod mvcc_stream;

mod path;

mod range;
mod table;

mod seqno;
mod slice;
mod slice_windows;

mod tree;

mod value;
mod value_type;
mod version;
use {
    boxed_iterator::BoxedIterator,
    key_range::KeyRange,
    table::{GlobalTableId, Table},
    value::InternalValue,
    value_type::ValueType,
};

pub use {
    cache::Cache,
    checksum::Checksum,
    compression::CompressionType,
    config::Config,
    descriptor_table::DescriptorTable,
    error::{Error, Result},
    slice::Slice,
    tree::{Tree, ingest::Ingestion},
};

use seqno::SequenceNumberCounter;
