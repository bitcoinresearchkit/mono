// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{cmp::Ordering, ops::Deref};

use rustc_hash::FxHashSet;

use super::{Choice, Input as CompactionInput};
use crate::{
    KeyRange,
    compaction::state::{CompactionState, hidden_set::HiddenSet},
    table::{Table, util::aggregate_run_key_range},
    version::{Level, Run, Version, run::Ranged},
};

fn bytes<'a>(tables: impl IntoIterator<Item = &'a Table>) -> u64 {
    tables.into_iter().map(Table::file_size).sum()
}

/// Tries to find the most optimal compaction set from one level into the other.
fn pick_minimal_compaction(
    curr_run: &Run<Table>,
    next_run: Option<&Run<Table>>,
    hidden_set: &HiddenSet,
) -> Option<(FxHashSet<u32>, bool)> {
    // NOTE: Find largest trivial move (if it exists)
    let largest_first = (1..=curr_run.len()).rev();
    if let Some(window) = largest_first
        .flat_map(|n| curr_run.windows(n))
        .find(|window| {
            if hidden_set.is_blocked(window.iter().map(Table::id)) {
                // IMPORTANT: Compaction is blocked because of other
                // on-going compaction
                return false;
            }

            let Some(next_run) = &next_run else {
                // No run in next level, so we can trivially move
                return true;
            };

            let key_range = aggregate_run_key_range(window);

            next_run.get_overlapping(&key_range).is_empty()
        })
    {
        let ids = window.iter().map(Table::id).collect();
        return Some((ids, true));
    }

    // NOTE: Look for merges
    let next_run = next_run?;

    // Each table with the next level's tables it overlaps, and the rest of this level within
    // their span, which comes along without another table rewritten. Starting from this level's
    // tables leaves none out: one that reaches past the next level's first or last key is merged
    // like any other, and widens that level.
    curr_run
        .iter()
        .filter_map(|table| {
            let overlapping = next_run.get_overlapping(table.key_range());
            let span = KeyRange::aggregate(
                std::iter::once(table.key_range()).chain(overlapping.iter().map(Table::key_range)),
            );
            let tables = || overlapping.iter().chain(curr_run.get_contained(&span));

            // IMPORTANT: Compaction is blocked because of other
            // on-going compaction
            (!hidden_set.is_blocked(tables().map(Table::id)))
                .then(|| (bytes(tables()), tables().map(Table::id).collect()))
        })
        // Find the compaction with the smallest write set
        .min_by_key(|(bytes, _)| *bytes)
        .map(|(_, ids)| (ids, false))
}

/// BRK's fixed leveled-compaction policy.
pub struct Strategy;

impl Strategy {
    /// Runs L0 holds before it's merged: a read checks every one of them.
    const L0_THRESHOLD: usize = 4;
    const TARGET_SIZE: u64 = 64 * 1_024 * 1_024;
    const LEVEL_RATIO: u64 = 10;

    /// Each level's target size, counted from the last level up: a `LEVEL_RATIO`th of the level
    /// below it, so the last level holds about nine tenths of the tree whatever its size, and a
    /// value waits for the tombstone that deletes it through a tenth of the data.
    ///
    /// A level whose target is under one table isn't in use (target 0): what it holds moves down.
    /// The last level takes whatever reaches it.
    fn level_targets(version: &Version) -> [u64; 7] {
        let mut targets = [0, 0, 0, 0, 0, 0, u64::MAX];
        let mut target = version
            .iter_levels()
            .last()
            .map_or(0, |level| bytes(level.iter().flat_map(|run| run.iter())));

        for slot in targets.iter_mut().skip(1).rev().skip(1) {
            target /= Self::LEVEL_RATIO;
            if target < Self::TARGET_SIZE {
                break;
            }
            *slot = target;
        }
        targets
    }

    /// Whether L0, due a merge, pays for one into the level under it, which rewrites the `below`
    /// bytes of that level; until then it's merged into one run of its own.
    ///
    /// Each merge of L0 into itself rewrites all it has gathered, so by the time it holds `l0`
    /// bytes, gathered `fresh` at a time, it has rewritten about `l0² / 2·fresh`. Going down
    /// rewrites `below` once. The bytes written per byte ingested are least where the two are
    /// equal: L0 goes down early over a small level and late over a large one, and a few KB per
    /// block gather for longer than a bulk ingestion, with no size configured for either.
    fn l0_goes_down(l0: &Level, below: u64) -> bool {
        let l0_bytes = bytes(l0.iter().flat_map(|run| run.iter()));
        // The oldest run is what earlier merges gathered; the others came since.
        let gathered = l0.iter().next_back().map_or(0, |run| bytes(run.iter()));
        let fresh = l0_bytes - gathered;

        u128::from(l0_bytes).pow(2) >= 2 * u128::from(fresh) * u128::from(below)
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::expect_used,
        clippy::too_many_lines,
        reason = "the asserted seven-level invariant guarantees every level exists and its index fits in u8"
    )]
    pub fn choose(version: &Version, state: &CompactionState) -> Choice {
        assert!(version.level_count() == 7, "should have exactly 7 levels");

        // Trivial move into Lmax
        'trivial_lmax: {
            let l0 = version.level(0).expect("first level should exist");

            if !l0.is_empty() && l0.is_disjoint() {
                let lmax_index = version.level_count() - 1;

                if (1..lmax_index)
                    .any(|idx| !version.level(idx).expect("level should exist").is_empty())
                {
                    // There are intermediary levels with data, cannot trivially move to Lmax
                    break 'trivial_lmax;
                }

                let lmax = version.level(lmax_index).expect("last level should exist");

                if !lmax
                    .aggregate_key_range()
                    .overlaps_with_key_range(&l0.aggregate_key_range())
                {
                    return Choice::Move(CompactionInput {
                        table_ids: l0.list_ids(),
                        dest_level: lmax_index as u8,
                        canonical_level: 1,
                        target_size: Self::TARGET_SIZE,
                    });
                }
            }
        }

        let targets = Self::level_targets(version);

        // Canonical L1, where L0 goes down to: the first level in use, or one above it that still
        // holds tables.
        let canonical_l1_idx = version.canonical_l1().min(
            targets
                .iter()
                .position(|&target| target > 0)
                .unwrap_or(usize::MAX),
        );

        // Number of levels we have to shift to get from the actual level idx to the canonical
        let level_shift = canonical_l1_idx - 1;

        // Trivial move into L1
        'trivial: {
            let first_level = version.l0();

            if first_level.run_count() == 1 {
                if version.level_is_busy(0, state.hidden_set())
                    || version.level_is_busy(canonical_l1_idx, state.hidden_set())
                {
                    break 'trivial;
                }

                let Some(target_level) = &version.level(canonical_l1_idx) else {
                    break 'trivial;
                };

                if target_level.run_count() != 1 {
                    break 'trivial;
                }

                let key_range = first_level.aggregate_key_range();

                // Get overlapping tables in next level
                let get_overlapping = target_level
                    .iter()
                    .flat_map(|run| run.get_overlapping(&key_range))
                    .map(Table::id)
                    .next();

                if get_overlapping.is_none() && first_level.is_disjoint() {
                    return Choice::Move(CompactionInput {
                        table_ids: first_level.list_ids(),
                        dest_level: canonical_l1_idx as u8,
                        canonical_level: 1,
                        target_size: Self::TARGET_SIZE,
                    });
                }
            }
        }

        // Scoring
        let mut scores = [0.0; 7];

        {
            // Score first level
            let first_level = version.l0();

            if first_level.run_count() >= Self::L0_THRESHOLD {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "precision loss is acceptable for scoring calculations"
                )]
                let ratio = (first_level.run_count() as f64) / (Self::L0_THRESHOLD as f64);
                scores[0] = ratio;
            }

            // Score L1+
            for (idx, level) in version.iter_levels().enumerate().skip(1) {
                if level.is_empty() {
                    continue;
                }

                let level_size = bytes(
                    level
                        .iter()
                        .flat_map(|x| x.iter())
                        // NOTE: Take bytes that are already being compacted into account,
                        // otherwise we may be overcompensating
                        .filter(|x| !state.hidden_set().is_hidden(x.id())),
                );

                // NOTE: We check for level length above
                #[expect(clippy::indexing_slicing)]
                let target_size = targets[idx];

                // NOTE: We check for level length above
                #[expect(clippy::indexing_slicing)]
                if level_size > target_size {
                    // A level that isn't in use scores infinity.
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "precision loss is acceptable for scoring calculations"
                    )]
                    let score = level_size as f64 / target_size as f64;
                    scores[idx] = score;

                    // NOTE: Force a trivial move
                    if version.level(idx + 1).is_some_and(Level::is_empty) {
                        scores[idx] = 99.99;
                    }
                }
            }
        }

        // Choose compaction
        #[expect(clippy::expect_used, reason = "highest score is expected to exist")]
        let (level_idx_with_highest_score, score) = scores
            .into_iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .expect("should have highest score somewhere");

        if score < 1.0 {
            return Choice::DoNothing;
        }

        // We choose L0 compaction
        if level_idx_with_highest_score == 0 {
            let first_level = version.l0();

            if version.level_is_busy(0, state.hidden_set()) {
                return Choice::DoNothing;
            }

            let Some(target_level) = &version.level(canonical_l1_idx) else {
                return Choice::DoNothing;
            };

            let mut table_ids = first_level.list_ids();

            let key_range = first_level.aggregate_key_range();

            // Get overlapping tables in next level
            let target_level_overlapping = target_level
                .iter()
                .flat_map(|run| run.get_overlapping(&key_range))
                .collect::<Vec<_>>();

            if !Self::l0_goes_down(first_level, bytes(target_level_overlapping.iter().copied())) {
                // Every L0 table, never some of them: `Version::with_merge` puts the merged run
                // behind all others of L0, where a point read expects the oldest records. A run
                // ingested during the merge is newer and stays ahead.
                return Choice::Merge(CompactionInput {
                    table_ids,
                    dest_level: 0,
                    canonical_level: 0,
                    target_size: Self::TARGET_SIZE,
                });
            }

            if version.level_is_busy(canonical_l1_idx, state.hidden_set()) {
                return Choice::DoNothing;
            }

            table_ids.extend(target_level_overlapping.iter().map(|table| table.id()));

            let choice = CompactionInput {
                table_ids,
                dest_level: canonical_l1_idx as u8,
                canonical_level: 1,
                target_size: Self::TARGET_SIZE,
            };

            if target_level_overlapping.is_empty() && first_level.is_disjoint() {
                return Choice::Move(choice);
            }
            return Choice::Merge(choice);
        }

        // We choose L1+ compaction instead

        // NOTE: Level count is 255 max
        let curr_level_index = level_idx_with_highest_score as u8;

        let next_level_index = curr_level_index + 1;

        let Some(level) = version.level(level_idx_with_highest_score) else {
            return Choice::DoNothing;
        };

        let Some(next_level) = version.level(next_level_index as usize) else {
            return Choice::DoNothing;
        };

        debug_assert!(level.is_disjoint(), "level should be disjoint");
        debug_assert!(next_level.is_disjoint(), "next level should be disjoint");

        #[expect(
            clippy::expect_used,
            reason = "first run should exist because score is >0.0"
        )]
        let Some((table_ids, can_trivial_move)) = pick_minimal_compaction(
            level.first_run().expect("should have exactly one run"),
            next_level.first_run().map(Deref::deref),
            state.hidden_set(),
        ) else {
            return Choice::DoNothing;
        };

        let choice = CompactionInput {
            table_ids,
            dest_level: next_level_index,
            canonical_level: next_level_index - (level_shift as u8),
            target_size: Self::TARGET_SIZE,
        };

        if can_trivial_move && level.is_disjoint() {
            return Choice::Move(choice);
        }
        Choice::Merge(choice)
    }
}
