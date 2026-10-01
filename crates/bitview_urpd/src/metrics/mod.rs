//! Derive, store and expose per-block URPD metrics.
pub(super) mod bounds;
mod cohort;
mod compute;
mod cost_basis;
pub(crate) mod density;
mod import;
mod metric_buckets;
mod price_distribution;
mod price_stats;

use bitview_cohort::AgeAggregate;
use bitview_traversable::Traversable;
use vecdb::{AnyStoredVec, Rw, StorageMode};

use crate::Replay;
use cohort::CohortMetrics;
use metric_buckets::MetricBuckets;

const WRITE_INTERVAL_BLOCKS: usize = 10_000;

#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    #[traversable(skip)]
    replay: M::WriteOnly<Replay>,
    #[traversable(skip)]
    buffer: M::WriteOnly<MetricBuckets>,
    #[traversable(flatten)]
    pub cohorts: AgeAggregate<CohortMetrics<M>>,
}

impl Metrics {
    fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cohorts
            .iter_mut()
            .flat_map(CohortMetrics::stored_vecs_mut)
    }
}
