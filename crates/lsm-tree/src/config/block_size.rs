// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::ops::Deref;

/// Block size policy
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockSizePolicy(Vec<u32>);

impl Deref for BlockSizePolicy {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BlockSizePolicy {
    /// Uses the last configured value for deeper levels.
    #[must_use]
    #[expect(
        clippy::indexing_slicing,
        reason = "constructors reject empty policies; the index is clamped"
    )]
    pub(crate) fn at_level(&self, level: usize) -> u32 {
        self.0[level.min(self.0.len() - 1)]
    }

    /// Uses the same block size in every level.
    #[must_use]
    pub fn all(c: u32) -> Self {
        Self(vec![c])
    }

    /// Constructs a custom block size policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy is empty or contains more than 255 elements.
    #[must_use]
    pub fn new(policy: impl Into<Vec<u32>>) -> Self {
        let policy = policy.into();
        assert!(!policy.is_empty(), "block-size policy may not be empty");
        assert!(policy.len() <= 255, "block-size policy is too large");
        Self(policy)
    }
}
