use bitview_cohort::{AgeRange, ByEpoch, Class};
use bitview_distribution::{metrics::CreationCohort, state::RealizedOps};
use bitview_primitives::CoinBlocks;
use bitview_vecs::{CachedSeries, PerBlockCumulativeRolling};
use brk_types::{Cents, CentsSats, Height};
use vecdb::WritableVec;

use crate::{RangeVecs, state::UTXOStates};

/// The replay loop can write origin metrics but cannot access address state.
pub struct OriginTargets<'a> {
    pub ranges: &'a mut AgeRange<RangeVecs>,
    pub epochs: &'a mut ByEpoch<CreationCohort>,
    pub classes: &'a mut Class<CreationCohort>,
    pub coinblocks_destroyed: &'a mut PerBlockCumulativeRolling<CoinBlocks>,
    pub all_capital: &'a mut CachedSeries<Height, Cents>,
}

impl OriginTargets<'_> {
    /// Writes every cohort's values once the block's spends and receives are applied, and the
    /// capital of all ranges together, summed exactly and converted to cents once.
    #[inline(always)]
    pub fn push(&mut self, states: &mut UTXOStates, price: Cents) {
        let mut all_capital = CentsSats::ZERO;
        for (vecs, state) in self.ranges.iter_mut().zip(states.age_range.iter_mut()) {
            vecs.push(state, price);
            all_capital += state.realized.cap_raw();
        }
        self.all_capital.push(all_capital.to_cents());
        for (vecs, state) in self.epochs.iter_mut().zip(states.epoch.iter_mut()) {
            vecs.push(state, price);
        }
        for (vecs, state) in self.classes.iter_mut().zip(states.class.iter_mut()) {
            vecs.push(state, price);
        }
    }
}
