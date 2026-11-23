use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Percentiles<T> {
    /// Uses the 95th percentile of the mode's supply-in-loss history.
    #[traversable(lead = "At its 95th percentile: losses as heavy as in the worst 5% of its history.")]
    pub p95: T,
    /// Uses the 98th percentile of the mode's supply-in-loss history.
    #[traversable(lead = "At its 98th percentile: losses as heavy as in the worst 2% of its history.")]
    pub p98: T,
    /// Uses the 99th percentile of the mode's supply-in-loss history.
    #[traversable(lead = "At its 99th percentile: losses as heavy as in the worst 1% of its history.")]
    pub p99: T,
    /// Uses the 99.5th percentile of the mode's supply-in-loss history.
    #[traversable(lead = "At its 99.5th percentile: losses as heavy as in the worst 0.5% of its history.")]
    pub p99_5: T,
    /// Uses the 99.9th percentile of the mode's supply-in-loss history.
    #[traversable(lead = "At its 99.9th percentile: losses as heavy as in the worst 0.1% of its history.")]
    pub p99_9: T,
}

collection!(LossPercentileId for Percentiles {
    P95 => p95,
    P98 => p98,
    P99 => p99,
    P99_5 => p99_5,
    P99_9 => p99_9,
});

/// The low tail of a mode's supply in profit: the same cut-offs as [`Percentiles`], read from the
/// profit side. Each is the complement of the loss threshold at the matching rank, so the floors
/// sit where the share in profit reaches it.
#[derive(Clone, Traversable)]
pub struct ProfitPercentiles<T> {
    /// The complement of the 95th-percentile supply-in-loss threshold.
    #[traversable(lead = "At its 5% mark: profits as thin as in the thinnest 5% of its history.")]
    pub p5: T,
    /// The complement of the 98th-percentile supply-in-loss threshold.
    #[traversable(lead = "At its 2% mark: profits as thin as in the thinnest 2% of its history.")]
    pub p2: T,
    /// The complement of the 99th-percentile supply-in-loss threshold.
    #[traversable(lead = "At its 1% mark: profits as thin as in the thinnest 1% of its history.")]
    pub p1: T,
    /// The complement of the 99.5th-percentile supply-in-loss threshold.
    #[traversable(lead = "At its 0.5% mark: profits as thin as in the thinnest 0.5% of its history.")]
    pub p0_5: T,
    /// The complement of the 99.9th-percentile supply-in-loss threshold.
    #[traversable(lead = "At its 0.1% mark: profits as thin as in the thinnest 0.1% of its history.")]
    pub p0_1: T,
}

collection!(ProfitPercentileId for ProfitPercentiles {
    P5 => p5,
    P2 => p2,
    P1 => p1,
    P0_5 => p0_5,
    P0_1 => p0_1,
});

impl ProfitPercentileId {
    /// The loss percentile whose threshold this one complements.
    pub const fn loss(self) -> LossPercentileId {
        match self {
            Self::P5 => LossPercentileId::P95,
            Self::P2 => LossPercentileId::P98,
            Self::P1 => LossPercentileId::P99,
            Self::P0_5 => LossPercentileId::P99_5,
            Self::P0_1 => LossPercentileId::P99_9,
        }
    }
}
