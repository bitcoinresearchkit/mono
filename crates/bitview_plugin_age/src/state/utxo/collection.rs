use bitview_cohort::{Age, AgeRange, ByHalvingEpoch, Vintage};
use bitview_distribution::walk::{Block, Origins};
use bitview_primitives::{CentsCompact, CostBasisSnapshot};
use brk_types::{Height, Sats};

use super::UTXOCohortState;
use crate::state::{CoreRealizedState, RealizedState, supply};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState>>,
    pub halving_epoch: ByHalvingEpoch<UTXOCohortState<CoreRealizedState>>,
    pub vintage: Vintage<UTXOCohortState<CoreRealizedState>>,
    /// The spot every cohort's split stands at: zero until the first block applied.
    spot: CentsCompact,
}

impl UTXOStates {
    pub fn new() -> Self {
        Self {
            age_range: AgeRange::from_fn(|_| UTXOCohortState::default()),
            halving_epoch: ByHalvingEpoch::from_fn(|_| UTXOCohortState::default()),
            vintage: Vintage::from_fn(|_| UTXOCohortState::default()),
            spot: CentsCompact::ZERO,
        }
    }

    /// Restores freshly constructed states from the origin balances. Restoring moves no
    /// block activity, so the states stay ready for the next block, whose spot every restored
    /// coin priced under it then crosses.
    pub fn restore_origins(&mut self, origins: &Origins<'_>) {
        let Some(last) = origins.amounts.len().checked_sub(1) else {
            return;
        };
        let timestamp = origins.timestamps[last];
        for (h, &amount) in origins.amounts.iter().enumerate() {
            let origin_timestamp = origins.timestamps[h];
            let snapshot = CostBasisSnapshot::from_utxo(origins.prices[h], &supply(amount));
            self.age_range
                .get_mut(Age::new(timestamp, origin_timestamp))
                .increment_snapshot(&snapshot);
            if let Some(state) = self.halving_epoch.get_mut(Height::from(h)) {
                state.increment_snapshot(&snapshot);
            }
            if let Some(state) = self.vintage.get_mut(origin_timestamp) {
                state.increment_snapshot(&snapshot);
            }
        }
    }

    /// Moves the spot to `block`'s, once the block's own changes are applied: what each origin
    /// the spot passed holds after the block changes side in its three cohorts.
    pub fn cross(&mut self, block: &Block<'_>) {
        let spot = CentsCompact::from(block.price());
        let timestamp = block.timestamp();
        for (price, origin) in block.order.passed(self.spot, spot) {
            // An origin past this block holds nothing yet.
            let Some(sats) = (block.amounts.get(origin)).map(|amount| Sats::new(amount.sats))
            else {
                continue;
            };
            if sats == Sats::ZERO {
                continue;
            }
            let origin_timestamp = block.timestamps[origin];
            self.age_range
                .get_mut(Age::new(timestamp, origin_timestamp))
                .cross(price, sats);
            if let Some(state) = self.halving_epoch.get_mut(Height::from(origin)) {
                state.cross(price, sats);
            }
            if let Some(state) = self.vintage.get_mut(origin_timestamp) {
                state.cross(price, sats);
            }
        }
        self.spot = spot;
    }

    pub fn reset_block(&mut self) {
        self.age_range
            .iter_mut()
            .for_each(|state| state.reset_block());
        self.halving_epoch
            .iter_mut()
            .for_each(|state| state.reset_block());
        self.vintage
            .iter_mut()
            .for_each(|state| state.reset_block());
    }
}
