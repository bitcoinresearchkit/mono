use bitview_cohort::AGE_RANGE_COUNT;

/// In-memory state that does not survive rollback.
#[derive(Clone, Default)]
pub struct UTXOTransientState {
    /// Cached positions for tick-tock boundary searches.
    pub tick_tock_cached_positions: [usize; AGE_RANGE_COUNT - 1],
}
