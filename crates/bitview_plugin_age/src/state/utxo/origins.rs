use bitview_cohort::Age;
use bitview_distribution::walk::Block;
use bitview_primitives::{CostBasisSnapshot, SupplyState};
use brk_types::{Cents, Height, Sats, Timestamp};
use tracing::error;

use crate::{
    compute::PriceRangeMax,
    state::{SendPrecomputed, UTXOStates, supply},
};

impl UTXOStates {
    /// Applies what `block` spent from each origin; returns the satoshi-blocks destroyed.
    pub fn send_origins(&mut self, block: &Block<'_>, price_range_max: &PriceRangeMax) -> u128 {
        let mut satblocks = 0;
        let send = block.height;
        let send_height = Height::from(send);
        let last_timestamp = block.timestamp();
        let current_price = block.price();

        for (origin, sent) in block.diff.spent() {
            let (receive_height, sent) = (Height::new(origin), supply(sent));
            let origin = origin as usize;
            satblocks += (send - origin) as u128 * u128::from(sent.value);
            let prev_price = block.prices[origin];
            let origin_timestamp = block.timestamps[origin];
            let age = Age::new(last_timestamp, origin_timestamp);

            // The table holds prices over `CentsCompact`'s ceiling at it, as the cohorts' splits
            // do: the peak is never below the send's own price, so the regret never goes negative.
            let peak_price = if sent.value == Sats::ZERO {
                Cents::ZERO
            } else {
                price_range_max
                    .range_max(receive_height, send_height)
                    .max(current_price)
            };

            // Pre-compute once for cohorts sharing the sent supply.
            if let Some(pre) =
                SendPrecomputed::new(&sent, current_price, prev_price, peak_price, age)
            {
                self.age_range
                    .get_mut(age)
                    .send_utxo_precomputed(&sent, &pre);
                if let Some(v) = self.halving_epoch.get_mut(receive_height) {
                    v.send_utxo_precomputed(&sent, &pre);
                }
                if let Some(v) = self.vintage.get_mut(origin_timestamp) {
                    v.send_utxo_precomputed(&sent, &pre);
                }
            } else if sent.utxo_count > 0 {
                // Zero-value outputs still contribute to spent-output counts.
                self.age_range.get_mut(age).send_utxo(
                    &sent,
                    current_price,
                    prev_price,
                    peak_price,
                    age,
                );
                if let Some(v) = self.halving_epoch.get_mut(receive_height) {
                    v.send_utxo(&sent, current_price, prev_price, peak_price, age);
                }
                if let Some(v) = self.vintage.get_mut(origin_timestamp) {
                    v.send_utxo(&sent, current_price, prev_price, peak_price, age);
                }
            }
        }
        satblocks
    }

    /// Removes an output that left the set without being spent (a BIP30-overwritten coinbase):
    /// supply, realized cap and cost basis drop, with no spend, volume, coin days or realized
    /// profit and loss.
    pub fn lose_origin(&mut self, receive_height: Height, lost: SupplyState, block: &Block<'_>) {
        let origin = usize::from(receive_height);
        let origin_timestamp = block.timestamps[origin];
        let age = Age::new(block.timestamp(), origin_timestamp);
        let snapshot = CostBasisSnapshot::from_utxo(block.prices[origin], &lost);
        self.age_range.get_mut(age).decrement_snapshot(&snapshot);
        if let Some(v) = self.halving_epoch.get_mut(receive_height) {
            v.decrement_snapshot(&snapshot);
        }
        if let Some(v) = self.vintage.get_mut(origin_timestamp) {
            v.decrement_snapshot(&snapshot);
        }
    }

    pub fn receive_origins(
        &mut self,
        supply: SupplyState,
        height: Height,
        timestamp: Timestamp,
        price: Cents,
    ) {
        let snapshot = CostBasisSnapshot::from_utxo(price, &supply);
        self.age_range.under_1h.increment_snapshot(&snapshot);
        match self.halving_epoch.get_mut(height) {
            Some(v) => v.increment_snapshot(&snapshot),
            None => error!("No halving epoch cohort for block {height}: its coins are in none"),
        }
        match self.vintage.get_mut(timestamp) {
            Some(v) => v.increment_snapshot(&snapshot),
            None => error!("No vintage cohort for block {height}'s year: its coins are in none"),
        }
    }
}
