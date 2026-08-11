use brk_traversable::Traversable;

use super::{AwakeVecs, DormantVecs};

#[derive(Clone, Traversable)]
pub struct CohortVecs {
    pub awake: AwakeVecs,
    pub dormant: DormantVecs,
}
