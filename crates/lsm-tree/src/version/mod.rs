// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod inner;
mod level;
mod optimize;
mod persist;
pub mod recovery;
pub mod run;
mod set;

pub use level::Level;
pub use run::Run;
pub use set::Set;

use crate::compaction::state::hidden_set::HiddenSet;
use crate::version::recovery::Recovery;
use crate::{Table, TableId};
use inner::Inner;
use optimize::optimize_runs;
use std::{ops::Deref, sync::Arc};

pub const DEFAULT_LEVEL_COUNT: u8 = 7;

/// Monotonically increasing ID of a version.
pub type VersionId = u64;

/// A version is an immutable, point-in-time view of a tree's structure
///
/// Any time a table is created or deleted, a new version is created.
#[derive(Clone)]
pub struct Version {
    inner: Arc<Inner>,
}

impl std::ops::Deref for Version {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// TODO: impl using generics so we can easily unit test Version transformation functions
impl Version {
    /// Returns the version ID.
    pub fn id(&self) -> VersionId {
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
    pub fn new(id: VersionId) -> Self {
        let levels = (0..DEFAULT_LEVEL_COUNT).map(|_| Level::empty()).collect();

        Self {
            inner: Arc::new(Inner { id, levels }),
        }
    }

    pub fn from_recovery(recovery: &Recovery, tables: &[Table]) -> crate::Result<Self> {
        let version_levels = recovery
            .table_ids
            .iter()
            .map(|level| {
                let level_runs = level
                    .iter()
                    .map(|run| {
                        let run_tables = run
                            .iter()
                            .map(|table| {
                                tables
                                    .iter()
                                    .find(|x| x.id() == table.id)
                                    .cloned()
                                    .ok_or(crate::Error::Unrecoverable)
                            })
                            .collect::<crate::Result<Vec<_>>>()?;

                        Ok(Arc::new(
                            #[expect(
                                clippy::expect_used,
                                reason = "empty runs should not exist, so there should not be any empty persisted runs"
                            )]
                            Run::new(run_tables).expect("persisted runs should not be empty"),
                        ))
                    })
                    .collect::<crate::Result<Vec<_>>>()?;

                Ok(Level::from_runs(level_runs))
            })
            .collect::<crate::Result<Vec<_>>>()?;

        Ok(Self::from_levels(recovery.curr_version_id, version_levels))
    }

    /// Creates a new pre-populated version.
    pub fn from_levels(id: VersionId, levels: Vec<Level>) -> Self {
        Self {
            inner: Arc::new(Inner { id, levels }),
        }
    }

    /// Returns the number of levels.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Returns an iterator through all levels.
    pub fn iter_levels(&self) -> impl Iterator<Item = &Level> {
        self.levels.iter()
    }

    /// Returns the number of tables in all levels.
    pub fn table_count(&self) -> usize {
        self.iter_levels().map(Level::table_count).sum()
    }

    /// Returns an iterator over all tables.
    pub fn iter_tables(&self) -> impl Iterator<Item = &Table> {
        self.levels
            .iter()
            .flat_map(Level::iter)
            .flat_map(|x| x.iter())
    }

    pub fn get_table(&self, id: TableId) -> Option<&Table> {
        self.iter_tables().find(|x| x.metadata.id == id)
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

        Self {
            inner: Arc::new(Inner { id, levels }),
        }
    }

    /// Returns a new version with a list of tables removed.
    ///
    /// The table files are not immediately deleted, this is handled by the version system's free list.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "preserves the fallible version-transformation API"
    )]
    pub fn with_dropped(&self, ids: &[TableId]) -> crate::Result<Self> {
        let id = self.id + 1;

        let mut levels = vec![];

        for level in &self.levels {
            let runs = level
                .runs
                .iter()
                .map(|run| {
                    // TODO: don't clone Arc inner if we don't need to modify
                    let mut run: Run<_> = run.deref().clone();

                    run.retain(|x| !ids.contains(&x.metadata.id));
                    run
                })
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>();

            let runs = optimize_runs(runs);

            levels.push(Level::from_runs(runs.into_iter().map(Arc::new).collect()));
        }

        Ok(Self {
            inner: Arc::new(Inner { id, levels }),
        })
    }

    pub fn with_merge(&self, old_ids: &[TableId], new_tables: &[Table], dest_level: usize) -> Self {
        let id = self.id + 1;

        let mut levels = vec![];

        for (level_idx, level) in self.levels.iter().enumerate() {
            let mut runs = level
                .runs
                .iter()
                .map(|run| {
                    // TODO: don't clone Arc inner if we don't need to modify
                    let mut run: Run<_> = run.deref().clone();
                    run.retain(|x| !old_ids.contains(&x.metadata.id));
                    run
                })
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>();

            if level_idx == dest_level
                && let Some(run) = Run::new(new_tables.to_vec())
            {
                runs.insert(0, run);
            }

            let runs = optimize_runs(runs);

            levels.push(Level::from_runs(runs.into_iter().map(Arc::new).collect()));
        }

        Self {
            inner: Arc::new(Inner { id, levels }),
        }
    }

    pub fn with_moved(&self, ids: &[TableId], dest_level: usize) -> Self {
        let id = self.id + 1;

        let affected_tables = self
            .iter_tables()
            .filter(|x| ids.contains(&x.id()))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(affected_tables.len(), ids.len(), "invalid table IDs");

        let mut levels = vec![];

        for (level_idx, level) in self.levels.iter().enumerate() {
            let mut runs = level
                .runs
                .iter()
                .map(|run| {
                    // TODO: don't clone Arc inner if we don't need to modify
                    let mut run: Run<_> = run.deref().clone();
                    run.retain(|x| !ids.contains(&x.metadata.id));
                    run
                })
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>();

            if level_idx == dest_level
                && let Some(run) = Run::new(affected_tables.clone())
            {
                runs.insert(0, run);
            }

            let runs = optimize_runs(runs);

            levels.push(Level::from_runs(runs.into_iter().map(Arc::new).collect()));
        }

        Self {
            inner: Arc::new(Inner { id, levels }),
        }
    }
}

impl Version {
    pub fn encode_into(&self, writer: &mut impl std::io::Write) -> Result<(), crate::Error> {
        use byteorder::{LittleEndian, WriteBytesExt};

        for level in self.iter_levels() {
            // Run count
            #[expect(
                clippy::cast_possible_truncation,
                reason = "there are always less than 256 runs"
            )]
            writer.write_u8(level.len() as u8)?;

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
                    writer.write_u128::<LittleEndian>(table.checksum().into_u128())?;
                    writer.write_u64::<LittleEndian>(table.global_seqno())?;
                }
            }
        }

        Ok(())
    }
}
