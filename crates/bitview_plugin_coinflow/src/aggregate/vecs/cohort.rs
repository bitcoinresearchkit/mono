use bitview_traversable::Traversable;

use super::{ImmobileVecs, MobileVecs};

#[derive(Clone, Traversable)]
pub struct CohortVecs {
    pub mobile: MobileVecs,
    pub immobile: ImmobileVecs,
}
