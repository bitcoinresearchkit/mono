use bitview_cohort::Age;
use bitview_primitives::{CostBasisSnapshot, SupplyState};
use brk_types::{Cents, Height, Sats, Timestamp};

use crate::{
    compute::ComputeContext,
    state::{SendPrecomputed, UTXOStates},
};

impl UTXOStates {
    /// Returns satoshi-blocks destroyed while applying each spent origin.
    pub fn send_origins(
        &mut self,
        height_to_sent: impl IntoIterator<Item = (Height, SupplyState)>,
        send_height: Height,
        ctx: &ComputeContext<'_>,
    ) -> u128 {
        let mut satblocks = 0;
        let send = usize::from(send_height);
        let last_timestamp = ctx.height_to_timestamp[send];
        let current_price = ctx.height_to_price[send];

        for (receive_height, sent) in height_to_sent {
            let origin = usize::from(receive_height);
            satblocks += (send - origin) as u128 * u128::from(sent.value);
            let prev_price = ctx.height_to_price[origin];
            let origin_timestamp = ctx.height_to_timestamp[origin];
            let age = Age::new(last_timestamp, origin_timestamp);

            let peak_price = if sent.value == Sats::ZERO {
                Cents::ZERO
            } else {
                ctx.price_range_max.range_max(receive_height, send_height)
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
    pub fn lose_origin(
        &mut self,
        receive_height: Height,
        lost: SupplyState,
        height: Height,
        ctx: &ComputeContext<'_>,
    ) {
        let origin = usize::from(receive_height);
        let origin_timestamp = ctx.height_to_timestamp[origin];
        let age = Age::new(
            ctx.height_to_timestamp[usize::from(height)],
            origin_timestamp,
        );
        let snapshot = CostBasisSnapshot::from_utxo(ctx.height_to_price[origin], &lost);
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
        if let Some(v) = self.halving_epoch.get_mut(height) {
            v.increment_snapshot(&snapshot);
        }
        if let Some(v) = self.vintage.get_mut(timestamp) {
            v.increment_snapshot(&snapshot);
        }
    }
}
