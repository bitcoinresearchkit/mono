use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{ImmobileVecs, MobileVecs};

#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    pub mobile: MobileVecs<M>,
    pub immobile: ImmobileVecs,
}
