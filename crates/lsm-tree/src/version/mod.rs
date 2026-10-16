// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod level;
pub mod optimize;
mod persist;
pub mod recovery;
pub mod run;
mod set;

pub use level::Level;
pub use run::Run;
pub use set::Set;

use crate::compaction::state::hidden_set::HiddenSet;
use crate::{Result, Table};
use byteorder::{LittleEndian, WriteBytesExt};
use optimize::optimize_runs;
use rustc_hash::FxHashSet;
use std::{
    io::{Error as IoError, ErrorKind, Write},
    ops::Deref,
    sync::Arc,
};

pub const DEFAULT_LEVEL_COUNT: u8 = 7;

fn level_contains_any(level: &Level, ids: &FxHashSet<u32>) -> bool {
    level
        .iter()
        .flat_map(|run| run.iter())
        .any(|table| ids.contains(&table.id()))
}

fn rebuild_level(
    level: &Level,
    removed_ids: &FxHashSet<u32>,
    new_run: Option<Run<Table>>,
    append_new_run: bool,
) -> Level {
    let mut runs = level
        .iter()
        .filter_map(|run| {
            Run::new(
                run.iter()
                    .filter(|table| !removed_ids.contains(&table.id()))
                    .cloned()
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

    if let Some(new_run) = new_run {
        if append_new_run {
            runs.push(new_run);
        } else {
            runs.insert(0, new_run);
        }
    }

    Level::from_runs(optimize_runs(runs).into_iter().map(Arc::new).collect())
}

/// A version is an immutable, point-in-time view of a tree's structure
///
/// Any time a table is created or deleted, a new version is created.
pub struct Version {
    id: u64,
    levels: Vec<Level>,
}

// TODO: impl using generics so we can easily unit test Version transformation functions
impl Version {
    /// Returns the version ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn l0(&self) -> &Level {
        #[expect(clippy::expect_used)]
        self.levels.first().expect("L0 should exist")
    }

    #[must_use]
    pub fn level_is_busy(&self, idx: usize, hidden_set: &HiddenSet) -> bool {
        self.level(idx).is_some_and(|level| {
            level
                .iter()
                .flat_map(|run| run.iter())
                .any(|table| hidden_set.is_hidden(table.id()))
        })
    }

    /// Creates a new empty version.
    pub fn new(id: u64) -> Self {
        let levels = (0..DEFAULT_LEVEL_COUNT).map(|_| Level::empty()).collect();

        Self { id, levels }
    }

    /// Creates a new pre-populated version.
    pub fn from_levels(id: u64, levels: Vec<Level>) -> Self {
        Self { id, levels }
    }

    /// Returns the number of levels.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Returns an iterator through all levels.
    pub fn iter_levels(&self) -> impl Iterator<Item = &Level> {
        self.levels.iter()
    }

    /// Returns an iterator over all tables.
    pub fn iter_tables(&self) -> impl Iterator<Item = &Table> {
        self.levels
            .iter()
            .flat_map(Level::iter)
            .flat_map(|x| x.iter())
    }

    /// Gets the n-th level.
    pub fn level(&self, n: usize) -> Option<&Level> {
        self.levels.get(n)
    }

    /// Creates a new version with the additional run added to the "top" of L0.
    pub fn with_new_l0_run(&self, run: &[Table]) -> Self {
        let id = self.id + 1;

        let mut levels = vec![];

        // L0
        levels.push({
            // Copy-on-write the first level with new run at top

            #[expect(clippy::expect_used, reason = "L0 always exists")]
            let l0 = self.levels.first().expect("L0 should always exist");

            let prev_runs = l0
                .runs
                .iter()
                .map(|run| {
                    let run: Run<_> = run.deref().clone();
                    run
                })
                .collect::<Vec<_>>();

            let mut runs = Vec::with_capacity(prev_runs.len() + 1);

            if let Some(run) = Run::new(run.to_vec()) {
                runs.push(run);
            }

            runs.extend(prev_runs);

            let runs = optimize_runs(runs);

            Level::from_runs(runs.into_iter().map(Arc::new).collect())
        });

        // L1+
        levels.extend(self.levels.iter().skip(1).cloned());

        Self { id, levels }
    }

    pub fn with_merge(
        &self,
        old_ids: &FxHashSet<u32>,
        new_tables: &[Table],
        dest_level: usize,
    ) -> Self {
        let id = self.id + 1;
        let levels = self
            .levels
            .iter()
            .enumerate()
            .map(|(level_idx, level)| {
                let is_destination = level_idx == dest_level && !new_tables.is_empty();

                if !is_destination && !level_contains_any(level, old_ids) {
                    return level.clone();
                }

                rebuild_level(
                    level,
                    old_ids,
                    is_destination.then(|| {
                        #[expect(
                            clippy::expect_used,
                            reason = "the destination run is known to contain tables"
                        )]
                        Run::new(new_tables.to_vec()).expect("new run should not be empty")
                    }),
                    // An intra-L0 result represents older inputs. Keep any run published while
                    // compaction was in flight ahead of it so point reads still see newest first.
                    dest_level == 0,
                )
            })
            .collect();

        Self { id, levels }
    }

    /// Moves the tables `ids` unchanged into `dest_level`.
    pub fn with_moved(&self, ids: &FxHashSet<u32>, dest_level: usize) -> Self {
        debug_assert!(dest_level > 0, "moves never target L0");
        let tables = self
            .iter_tables()
            .filter(|table| ids.contains(&table.id()))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(tables.len(), ids.len(), "invalid table IDs");

        self.with_merge(ids, &tables, dest_level)
    }
}

impl Version {
    pub fn encode_into(&self, writer: &mut impl Write) -> Result<()> {
        for level in self.iter_levels() {
            // Run count, a byte: a level with more runs is refused, not truncated.
            let runs = u8::try_from(level.len()).map_err(|_| {
                IoError::new(ErrorKind::InvalidData, "a level holds more than 255 runs")
            })?;
            writer.write_u8(runs)?;

            for run in level.iter() {
                // Table count
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "there are always less than 4 billion tables in a run"
                )]
                writer.write_u32::<LittleEndian>(run.len() as u32)?;

                // Tables
                for table in run.iter() {
                    writer.write_u32::<LittleEndian>(table.id())?;
                    writer.write_u64::<LittleEndian>(table.global_seqno())?;
                }
            }
        }

        Ok(())
    }
}
