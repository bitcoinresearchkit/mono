use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Percentiles<T> {
    /// Uses the 95th percentile of the mode's supply-in-loss history.
    pub p95: T,
    /// Uses the 98th percentile of the mode's supply-in-loss history.
    pub p98: T,
    /// Uses the 99th percentile of the mode's supply-in-loss history.
    pub p99: T,
    /// Uses the 99.5th percentile of the mode's supply-in-loss history.
    pub p99_5: T,
    /// Uses the 99.9th percentile of the mode's supply-in-loss history.
    pub p99_9: T,
}

collection!(LossPercentileId for Percentiles {
    P95 => p95,
    P98 => p98,
    P99 => p99,
    P99_5 => p99_5,
    P99_9 => p99_9,
});
