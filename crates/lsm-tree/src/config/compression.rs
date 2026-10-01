// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::ops::Deref;

use crate::CompressionType;

/// Compression policy
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompressionPolicy(Vec<CompressionType>);

impl Deref for CompressionPolicy {
    type Target = [CompressionType];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CompressionPolicy {
    /// Uses the last configured value for deeper levels.
    #[must_use]
    #[expect(
        clippy::indexing_slicing,
        reason = "constructors reject empty policies; the index is clamped"
    )]
    pub(crate) fn at_level(&self, level: usize) -> CompressionType {
        self.0[level.min(self.0.len() - 1)]
    }

    /// Uses the same compression in every level.
    #[must_use]
    pub fn all(c: CompressionType) -> Self {
        Self(vec![c])
    }

    /// Constructs a custom compression policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy is empty or contains more than 255 elements.
    #[must_use]
    pub fn new(policy: impl Into<Vec<CompressionType>>) -> Self {
        let policy = policy.into();
        assert!(!policy.is_empty(), "compression policy may not be empty");
        assert!(policy.len() <= 255, "compression policy is too large");
        Self(policy)
    }
}
