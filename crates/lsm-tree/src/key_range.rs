// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::Slice;

/// A key range in the format of [min, max] (inclusive on both sides)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRange(Slice, Slice);

impl KeyRange {
    /// Creates a new key range.
    #[must_use]
    pub fn new((min, max): (Slice, Slice)) -> Self {
        Self(min, max)
    }

    /// Creates an empty key range.
    #[must_use]
    pub fn empty() -> Self {
        Self(Slice::empty(), Slice::empty())
    }

    /// Returns the lower bound.
    #[must_use]
    pub fn min(&self) -> &Slice {
        &self.0
    }

    /// Returns the upper bound.
    #[must_use]
    pub fn max(&self) -> &Slice {
        &self.1
    }

    fn as_tuple(&self) -> (&Slice, &Slice) {
        (self.min(), self.max())
    }

    /// Returns `true` if the `other` is fully contained in this range.
    #[must_use]
    pub fn contains_range(&self, other: &Self) -> bool {
        let (start1, end1) = self.as_tuple();
        let (start2, end2) = other.as_tuple();
        start1 <= start2 && end1 >= end2
    }

    /// Returns `true` if the `other` overlaps at least partially with this range.
    #[must_use]
    pub fn overlaps_with_key_range(&self, other: &Self) -> bool {
        let (start1, end1) = self.as_tuple();
        let (start2, end2) = other.as_tuple();
        end1 >= start2 && start1 <= end2
    }

    /// Aggregates a key range.
    pub fn aggregate<'a>(mut iter: impl Iterator<Item = &'a Self>) -> Self {
        let Some(first) = iter.next() else {
            return Self::empty();
        };

        let mut min = first.min();
        let mut max = first.max();

        for other in iter {
            let x = other.min();
            if x < min {
                min = x;
            }

            let x = other.max();
            if x > max {
                max = x;
            }
        }

        Self(min.clone(), max.clone())
    }
}
