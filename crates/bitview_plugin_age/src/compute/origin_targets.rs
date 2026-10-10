use bitview_cohort::{AgeRange, ByEpoch, Class};
use bitview_primitives::CoinBlocks;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::Cents;

use crate::{CohortVecs, RangeVecs, state::UTXOStates};

/// The replay loop can write origin metrics but cannot access address state.
pub struct OriginTargets<'a> {
    pub ranges: &'a mut AgeRange<RangeVecs>,
    pub epochs: &'a mut ByEpoch<CohortVecs>,
    pub classes: &'a mut Class<CohortVecs>,
    pub coinblocks_destroyed: &'a mut PerBlockCumulativeRolling<CoinBlocks>,
}

impl OriginTargets<'_> {
    /// Writes every cohort's values once the block's spends and receives are applied.
    #[inline(always)]
    pub fn push(&mut self, states: &mut UTXOStates, price: Cents) {
        for (vecs, state) in self.ranges.iter_mut().zip(states.age_range.iter_mut()) {
            vecs.push(state, price);
        }
        for (vecs, state) in self.epochs.iter_mut().zip(states.epoch.iter_mut()) {
            vecs.push(state, price);
        }
        for (vecs, state) in self.classes.iter_mut().zip(states.class.iter_mut()) {
            vecs.push(state, price);
        }
    }
}
