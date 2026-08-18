//! BRK's table-only LSM database.
//!
//! Writes are sorted and ingested directly into immutable `SSTables`. The crate
//! deliberately has no write-ahead log, memtable API, snapshots, or general
//! point-write API because BRK does not use them.

#![deny(unsafe_code)]
#![deny(clippy::all, clippy::cargo, missing_docs)]
#![allow(
    clippy::cargo_common_metadata,
    reason = "not every internal workspace package is independently published"
)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::indexing_slicing)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "poisoned locks and violated internal invariants are unrecoverable"
)]
#![allow(clippy::missing_const_for_fn, clippy::significant_drop_tightening)]

mod builder;
mod database;
mod db_config;
mod error;
mod file;
mod keyspace;
mod locked_file;
mod worker_pool;

/// LSM storage policies exposed to BRK's store profiles.
pub mod config {
    pub use lsm_tree::config::{
        BloomConstructionPolicy, FilterPolicy, FilterPolicyEntry, PartitioningPolicy,
        PinningPolicy, RestartIntervalPolicy,
    };
}

pub use {
    builder::Builder as DatabaseBuilder,
    database::Database,
    error::{Error, Result},
    keyspace::{Keyspace, options::CreateOptions as KeyspaceCreateOptions},
};
