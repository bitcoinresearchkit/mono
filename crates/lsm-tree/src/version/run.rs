// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::ops::{Bound, Deref, RangeBounds};

use crate::{KeyRange, Slice};

pub trait Ranged {
    fn key_range(&self) -> &KeyRange;
}

/// A disjoint run of tables
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Run<T: Ranged>(Vec<T>);

impl<T: Ranged> Deref for Run<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Ranged> Run<T> {
    pub fn new(items: Vec<T>) -> Option<Self> {
        if items.is_empty() {
            None
        } else {
            Some(Self(items))
        }
    }

    pub fn inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Returns the table that may possibly contains the given key.
    pub fn get_for_key(&self, key: &[u8]) -> Option<&T> {
        let idx = self.partition_point(|x| x.key_range().max() < &key);

        self.0.get(idx).filter(|x| x.key_range().min() <= &key)
    }

    /// Returns the run's key range.
    pub fn aggregate_key_range(&self) -> KeyRange {
        #[expect(clippy::expect_used, reason = "by definition, runs are never empty")]
        let lo = self.first().expect("run should never be empty");

        #[expect(clippy::expect_used, reason = "by definition, runs are never empty")]
        let hi = self.last().expect("run should never be empty");

        KeyRange::new((lo.key_range().min().clone(), hi.key_range().max().clone()))
    }

    /// Returns the sub slice of tables in the run that have
    /// a key range overlapping the input key range.
    pub fn get_overlapping<'a>(&'a self, key_range: &'a KeyRange) -> &'a [T] {
        let range = key_range.min()..=key_range.max();

        let Some((lo, hi)) = self.range_overlap_indexes::<Slice, _>(&range) else {
            return &[];
        };

        self.get(lo..=hi).unwrap_or_default()
    }

    /// Returns the sub slice of tables of tables in the run that have
    /// a key range fully contained in the input key range.
    pub fn get_contained<'a>(&'a self, key_range: &KeyRange) -> &'a [T] {
        // Disjoint and sorted: both bounds ascend across the run.
        let lo = self.partition_point(|x| x.key_range().min() < key_range.min());
        let hi = self.partition_point(|x| x.key_range().max() <= key_range.max());
        self.get(lo..hi).unwrap_or_default()
    }

    /// Returns the indexes of the interval [min, max] of tables that overlap with a given range.
    pub fn range_overlap_indexes<K: AsRef<[u8]>, R: RangeBounds<K>>(
        &self,
        key_range: &R,
    ) -> Option<(usize, usize)> {
        let level = &self.0;

        let lo = match key_range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(start_key) => {
                level.partition_point(|x| x.key_range().max() < start_key)
            }
            Bound::Excluded(start_key) => {
                level.partition_point(|x| x.key_range().max() <= start_key)
            }
        };

        if lo >= level.len() {
            return None;
        }

        // NOTE: We check for level length above
        #[expect(clippy::indexing_slicing)]
        let truncated_level = &level[lo..];

        let hi = match key_range.end_bound() {
            Bound::Unbounded => level.len() - 1,
            Bound::Included(end_key) => {
                // IMPORTANT: We need to add back `lo` because we sliced it off
                let idx = lo + truncated_level.partition_point(|x| x.key_range().min() <= end_key);

                if idx == 0 {
                    return None;
                }

                idx.saturating_sub(1) // To avoid underflow
            }
            Bound::Excluded(end_key) => {
                // IMPORTANT: We need to add back `lo` because we sliced it off
                let idx = lo + truncated_level.partition_point(|x| x.key_range().min() < end_key);

                if idx == 0 {
                    return None;
                }

                idx.saturating_sub(1) // To avoid underflow
            }
        };

        if lo > hi {
            return None;
        }

        Some((lo, hi))
    }
}
