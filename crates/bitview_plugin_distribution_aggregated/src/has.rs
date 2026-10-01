use crate::Vecs;
use vecdb::StorageMode;
pub trait HasDistributionAggregated<M: StorageMode> {
    fn distribution_aggregated(&self) -> &Vecs<M>;
}
