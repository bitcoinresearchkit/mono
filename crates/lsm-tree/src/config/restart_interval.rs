// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::ops::Deref;

/// Restart interval policy
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RestartIntervalPolicy(Vec<u8>);

impl Deref for RestartIntervalPolicy {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RestartIntervalPolicy {
    /// Uses the last configured value for deeper levels.
    #[must_use]
    #[expect(
        clippy::indexing_slicing,
        reason = "constructors reject empty policies; the index is clamped"
    )]
    pub(crate) fn at_level(&self, level: usize) -> u8 {
        self.0[level.min(self.0.len() - 1)]
    }

    // TODO: accept Vec... Into<Vec<...>>? or owned

    /// Uses the same block size in every level.
    #[must_use]
    pub fn all(c: u8) -> Self {
        Self(vec![c])
    }

    /// Constructs a custom block size policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy is empty or contains more than 255 elements.
    #[must_use]
    pub fn new(policy: impl Into<Vec<u8>>) -> Self {
        let policy = policy.into();
        assert!(
            !policy.is_empty(),
            "restart-interval policy may not be empty"
        );
        assert!(policy.len() <= 255, "restart-interval policy is too large");
        Self(policy)
    }
}
