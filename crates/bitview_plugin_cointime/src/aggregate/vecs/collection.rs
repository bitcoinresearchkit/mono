use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{CohortVecs, Sources};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub all: CohortVecs,
    pub sth: CohortVecs,
    pub lth: CohortVecs,
    pub sources: Sources<M>,
}
