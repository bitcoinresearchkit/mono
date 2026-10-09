use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentVec, PerBlockRolling};
use brk_types::Weight;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    pub weight: PerBlockRolling<Weight, M>,
    /// Block weight as a share of the 4,000,000-weight-unit consensus limit.
    /// 100% means the block reached the limit; lower values indicate unused
    /// weight capacity.
    pub fullness: LazyPercentVec<PartsPerMillion32, Weight>,
}
