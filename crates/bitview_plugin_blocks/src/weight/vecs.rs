use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFixedRatioVec, PerBlockRolling};
use brk_types::Weight;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    pub weight: PerBlockRolling<Weight, M>,
    /// Block weight divided by the 4,000,000-weight-unit consensus limit. A
    /// value of one means the block reached the limit; lower values indicate
    /// unused weight capacity.
    pub fullness: LazyFixedRatioVec<PartsPerMillion32, Weight>,
}
