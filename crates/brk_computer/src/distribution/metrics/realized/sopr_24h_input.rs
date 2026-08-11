use brk_types::{Cents, Height};
use vecdb::{DeltaSub, LazyDeltaVec};

use crate::internal::{LazyFiatPerBlockCumulativeWithSums, LazyValuePerBlockCumulativeRolling};

#[derive(Clone)]
pub(crate) struct Sopr24hInput {
    pub(super) transfer_volume: LazyDeltaVec<Height, Cents, Cents, DeltaSub>,
    pub(super) value_destroyed: LazyDeltaVec<Height, Cents, Cents, DeltaSub>,
}

impl Sopr24hInput {
    pub(crate) fn new(
        transfer_volume: &LazyValuePerBlockCumulativeRolling,
        value_destroyed: &LazyFiatPerBlockCumulativeWithSums<Cents>,
    ) -> Self {
        Self {
            transfer_volume: transfer_volume.sum._24h.cents.height.clone(),
            value_destroyed: value_destroyed.sum._24h.cents.height.clone(),
        }
    }
}
