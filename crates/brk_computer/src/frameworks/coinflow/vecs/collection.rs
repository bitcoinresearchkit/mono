use brk_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{AgeRangeVecs, AggregateSources, AggregateVecs};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub age_range: AgeRangeVecs<M>,
    #[traversable(flatten)]
    pub all: AggregateVecs,
    pub sth: AggregateVecs,
    pub lth: AggregateVecs,
    pub aggregate_sources: AggregateSources<M>,
}
