use bitview_primitives::{PartsPerMillion32, StoredU64, Weight64};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockRolling, LazyPercentVec};
use brk_types::Weight;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    pub weight: LazyPerBlockRolling<Weight64, StoredU64>,
    /// Block weight divided by the 4,000,000-weight-unit consensus limit. A
    /// value of one means the block reached the limit; lower values indicate
    /// unused weight capacity.
    pub fullness: LazyPercentVec<PartsPerMillion32, Weight>,
}
