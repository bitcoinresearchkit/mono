use crate::{
    compute::ComputeContext,
    state::{SendPrecomputed, UTXOStates},
};
use brk_types::{Age, Cents, CostBasisSnapshot, Height, Sats, SupplyState, Timestamp};
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
        let last_timestamp = ctx.height_to_timestamp[usize::from(send_height)];
        let current_price = ctx.height_to_price[usize::from(send_height)];

        for (receive_height, sent) in height_to_sent {
            let origin = usize::from(receive_height);
            satblocks += (send - origin) as u128 * u128::from(sent.value);
            let prev_price = ctx.height_to_price[origin];
            let origin_timestamp = ctx.height_to_timestamp[origin];
            let age = Age::new(last_timestamp, origin_timestamp);

            // Compute peak price during holding period for peak regret
            // This is the max price between receive and send heights
            let peak_price = if sent.value == Sats::ZERO {
                Cents::ZERO
            } else {
                ctx.price_range_max.max_between(receive_height, send_height)
            };

            // Pre-compute once for cohorts sharing the sent supply.
            if let Some(pre) =
                SendPrecomputed::new(&sent, current_price, prev_price, peak_price, age)
            {
                self.age_range
                    .get_mut(age)
                    .send_utxo_precomputed(&sent, &pre);
                if let Some(v) = self.epoch.mut_vec_from_height(receive_height) {
                    v.send_utxo_precomputed(&sent, &pre);
                }
                if let Some(v) = self.class.mut_vec_from_timestamp(origin_timestamp) {
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
                if let Some(v) = self.epoch.mut_vec_from_height(receive_height) {
                    v.send_utxo(&sent, current_price, prev_price, peak_price, age);
                }
                if let Some(v) = self.class.mut_vec_from_timestamp(origin_timestamp) {
                    v.send_utxo(&sent, current_price, prev_price, peak_price, age);
                }
            }
        }
        satblocks
    }
    pub fn receive_origins(
        &mut self,
        supply: SupplyState,
        height: Height,
        timestamp: Timestamp,
        price: Cents,
    ) {
        let snapshot = CostBasisSnapshot::from_utxo(price, &supply);
        self.age_range
            .under_1h
            .receive_utxo_snapshot(&supply, &snapshot);
        if let Some(v) = self.epoch.mut_vec_from_height(height) {
            v.receive_utxo_snapshot(&supply, &snapshot);
        }
        if let Some(v) = self.class.mut_vec_from_timestamp(timestamp) {
            v.receive_utxo_snapshot(&supply, &snapshot);
        }
    }
}

#[cfg(test)]
#[path = "origin_tests.rs"]
mod tests;
