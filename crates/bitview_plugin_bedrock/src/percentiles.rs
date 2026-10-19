use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Percentiles<T> {
    /// Uses the 95th percentile of the mode's supply-in-loss history.
    pub pct95: T,
    /// Uses the 98th percentile of the mode's supply-in-loss history.
    pub pct98: T,
    /// Uses the 99th percentile of the mode's supply-in-loss history.
    pub pct99: T,
    /// Uses the 99.5th percentile of the mode's supply-in-loss history.
    pub pct99_5: T,
    /// Uses the 99.9th percentile of the mode's supply-in-loss history.
    pub pct99_9: T,
}

collection!(LossPercentileId for Percentiles {
    Pct95 => pct95,
    Pct98 => pct98,
    Pct99 => pct99,
    Pct99_5 => pct99_5,
    Pct99_9 => pct99_9,
});
