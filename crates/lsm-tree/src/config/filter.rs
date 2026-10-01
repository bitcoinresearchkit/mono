// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::ops::Deref;

pub use crate::table::filter::BloomConstructionPolicy;

/// Filter policy entry
///
/// Each level can be configured with a different filter type and bits per key
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum FilterPolicyEntry {
    /// Skip filter construction
    None,

    /// Standard bloom filter with K bits per key
    Bloom(BloomConstructionPolicy),
}

/// Filter policy
#[derive(Debug, Clone, PartialEq)]
pub struct FilterPolicy(Vec<FilterPolicyEntry>);

impl Deref for FilterPolicy {
    type Target = [FilterPolicyEntry];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FilterPolicy {
    /// Uses the last configured value for deeper levels.
    #[must_use]
    #[expect(
        clippy::indexing_slicing,
        reason = "constructors reject empty policies; the index is clamped"
    )]
    pub(crate) fn at_level(&self, level: usize) -> FilterPolicyEntry {
        self.0[level.min(self.0.len() - 1)]
    }

    /// Disables all filters.
    ///
    /// **Not recommended unless you know what you are doing!**
    #[must_use]
    pub fn disabled() -> Self {
        Self::all(FilterPolicyEntry::None)
    }

    /// Uses the same block size in every level.
    #[must_use]
    pub(crate) fn all(c: FilterPolicyEntry) -> Self {
        Self(vec![c])
    }

    /// Constructs a custom block size policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy is empty or contains more than 255 elements.
    #[must_use]
    pub fn new(policy: impl Into<Vec<FilterPolicyEntry>>) -> Self {
        let policy = policy.into();
        assert!(!policy.is_empty(), "filter policy may not be empty");
        assert!(policy.len() <= 255, "filter policy is too large");
        Self(policy)
    }
}
