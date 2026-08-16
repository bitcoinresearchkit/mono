use brk_traversable::Traversable;
use brk_types::{CapitalSentimentPhase, StoredBool, StoredI8, StoredU8};
use vecdb::{Rw, StorageMode};

use crate::internal::{DailyMetric, LazyDailyMetric};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Compact daily source of truth.
    #[traversable(hidden)]
    pub(super) phase_code: DailyMetric<StoredU8, M>,

    /// Whether the stateful capital-sentiment strategy is long for the indexed
    /// day. It enters when spot crosses from below to at or above the STH
    /// capitalized price and exits when the phase is classified as sell.
    pub is_long: DailyMetric<StoredBool, M>,
    /// Whether the stateful capital-sentiment strategy is short for the indexed
    /// day; exactly the complement of `is_long`.
    pub is_short: LazyDailyMetric<StoredBool, StoredBool>,
    pub phase: LazyDailyMetric<Option<CapitalSentimentPhase>, StoredU8>,
    pub score: LazyDailyMetric<Option<StoredI8>, Option<CapitalSentimentPhase>>,
}
