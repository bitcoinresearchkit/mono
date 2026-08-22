use brk_types::{Cents, Height};
use vecdb::{DeltaSub, LazyDeltaVec};

use bitview_compute::{LazyFiatPerBlockCumulativeRolling, LazyValuePerBlockCumulativeRolling};

#[derive(Clone)]
pub struct Sopr24hInput {
    pub transfer_volume: LazyDeltaVec<Height, Cents, Cents, DeltaSub>,
    pub value_destroyed: LazyDeltaVec<Height, Cents, Cents, DeltaSub>,
}

impl Sopr24hInput {
    pub fn new(
        transfer_volume: &LazyValuePerBlockCumulativeRolling,
        value_destroyed: &LazyFiatPerBlockCumulativeRolling<Cents>,
    ) -> Self {
        Self {
            transfer_volume: transfer_volume.sum._24h.cents.height.clone(),
            value_destroyed: value_destroyed.sum._24h.cents.height.clone(),
        }
    }
}
