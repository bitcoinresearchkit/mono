/// Per-level policy controlling index and filter partitioning.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PartitioningPolicy(Vec<bool>);

impl PartitioningPolicy {
    /// Returns whether the selected level should be partitioned.
    #[must_use]
    pub fn get(&self, level: usize) -> bool {
        self.0
            .get(level)
            .copied()
            .unwrap_or_else(|| self.0.last().copied().unwrap_or(false))
    }

    /// Uses the same policy in every level.
    #[must_use]
    pub fn all(partition: bool) -> Self {
        Self(vec![partition])
    }

    /// Fully disables partitioning.
    #[must_use]
    pub fn disabled() -> Self {
        Self::all(false)
    }

    /// Constructs a custom policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy is empty or contains more than 255 elements.
    #[must_use]
    pub fn new(policy: impl Into<Vec<bool>>) -> Self {
        let policy = policy.into();
        assert!(!policy.is_empty(), "partitioning policy may not be empty");
        assert!(policy.len() <= 255, "partitioning policy is too large");
        Self(policy)
    }
}
