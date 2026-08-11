use brk_cohort::AGE_RANGE_COUNT;

use super::fenwick::CostBasisFenwick;

/// In-memory state that does not survive rollback.
#[derive(Clone, Default)]
pub(crate) struct UTXOTransientState {
    pub(super) fenwick: CostBasisFenwick,
    /// Cached positions for tick-tock boundary searches.
    pub(super) tick_tock_cached_positions: [usize; AGE_RANGE_COUNT - 1],
}
