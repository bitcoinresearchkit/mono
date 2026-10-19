use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{AwakeVecs, DormantVecs};

#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    pub awake: AwakeVecs<M>,
    pub dormant: DormantVecs,
}
