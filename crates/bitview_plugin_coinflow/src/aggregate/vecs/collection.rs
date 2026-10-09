use bitview_cohort::AgeAggregate;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{CohortVecs, Sources};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: AgeAggregate<CohortVecs>,
    #[traversable(hidden)]
    pub sources: Sources<M>,
}
