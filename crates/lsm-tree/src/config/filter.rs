// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

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

super::level_policy!(
    /// Filter construction, per level.
    FilterPolicy(FilterPolicyEntry),
    "filter"
);

impl FilterPolicy {
    /// Disables all filters.
    ///
    /// **Not recommended unless you know what you are doing!**
    #[must_use]
    pub fn disabled() -> Self {
        Self::all(FilterPolicyEntry::None)
    }

    /// The Bloom policy of `level`, if it builds filters.
    pub(crate) fn bloom_at(&self, level: usize) -> Option<BloomConstructionPolicy> {
        match self.at_level(level) {
            FilterPolicyEntry::Bloom(policy) if policy.is_active() => Some(policy),
            FilterPolicyEntry::Bloom(_) | FilterPolicyEntry::None => None,
        }
    }
}
