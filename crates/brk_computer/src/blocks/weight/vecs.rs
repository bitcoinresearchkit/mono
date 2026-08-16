use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredU64, Weight, Weight64};

use crate::internal::{LazyPerBlockRolling, LazyPercentVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    pub weight: LazyPerBlockRolling<Weight64, StoredU64>,
    /// Block weight divided by the 4,000,000-weight-unit consensus limit.
    pub fullness: LazyPercentVec<PartsPerMillion32, Weight>,
}
