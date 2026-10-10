use bitview_cohort::AgeAggregate;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{CohortVecs, Sources};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub cohorts: AgeAggregate<CohortVecs<M>>,
    #[traversable(hidden)]
    pub sources: Sources<M>,
}
