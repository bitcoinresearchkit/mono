// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

//! Contains compaction strategies

mod flavour;
pub mod leveled;
pub mod state;
pub mod stream;
pub mod worker;

use crate::{HashSet, TableId};

/// Input for compactor
///
/// The compaction strategy chooses which tables to compact and how.
/// That information is given to the compactor.
#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    /// Tables to compact
    pub table_ids: HashSet<TableId>,

    /// Level to put the created tables into
    pub dest_level: u8,

    /// The logical level the tables are part of
    pub canonical_level: u8,

    /// Table target size
    ///
    /// If a table merge reaches the size threshold, a new table is started.
    /// This results in a sorted "run" of tables.
    pub target_size: u64,
}

/// Describes what to do (compact or not)
#[derive(Debug, Eq, PartialEq)]
pub enum Choice {
    /// Just do nothing.
    DoNothing,

    /// Moves tables into another level without rewriting.
    Move(Input),

    /// Compacts some tables into a new level.
    Merge(Input),
}
