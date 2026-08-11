use brk_cohort::{AGE_BOUNDARIES, AGE_RANGE_IDS, AgeRange};
use brk_types::{CostBasisSnapshot, ONE_HOUR_IN_SEC, Sats, Timestamp};
use vecdb::unlikely;

use crate::distribution::state::BlockState;

use super::UTXOStates;

impl UTXOStates {
    /// Handle age transitions when processing a new block.
    ///
    /// UTXOs age with each block. When they cross hour boundaries,
    /// they move between age-based cohorts (e.g., from "0-1h" to "1h-1d").
    ///
    /// Uses cached positions per boundary to avoid binary search.
    /// Since timestamps are monotonic, positions only advance forward.
    /// Complexity: O(k * c), where k is the boundary count and c is ~1 forward scan step.
    ///
    /// Returns how many sats matured OUT OF each cohort into the older adjacent one.
    /// `over_15y` is always zero since nothing ages out of the oldest cohort.
    pub(crate) fn tick_tock_next_block(
        &mut self,
        chain_state: &[BlockState],
        timestamp: Timestamp,
    ) -> AgeRange<Sats> {
        if chain_state.is_empty() {
            return AgeRange::default();
        }

        let prev_timestamp = chain_state.last().unwrap().timestamp;
        let elapsed = (*timestamp).saturating_sub(*prev_timestamp);

        // Skip if no time has passed
        if elapsed == 0 {
            return AgeRange::default();
        }

        let mut matured = AgeRange::default();
        let Self {
            age_range,
            transient,
            ..
        } = self;
        let cached = &mut transient.tick_tock_cached_positions;

        // For each boundary (in hours), find blocks that just crossed it
        for (boundary_idx, (&boundary_hours, adjacent)) in AGE_BOUNDARIES
            .iter()
            .zip(AGE_RANGE_IDS.windows(2))
            .enumerate()
        {
            let boundary_seconds = (boundary_hours as u32) * ONE_HOUR_IN_SEC;

            // Blocks crossing boundary B have timestamps in (prev - B*HOUR, curr - B*HOUR]
            // prev_hours < B and curr_hours >= B
            // means: block was younger than B hours, now is B hours or older
            let upper_timestamp = (*timestamp).saturating_sub(boundary_seconds);
            let lower_timestamp = (*prev_timestamp).saturating_sub(boundary_seconds);

            // Skip if the range is empty (would happen if boundary > chain age)
            if upper_timestamp <= lower_timestamp {
                continue;
            }

            // Find start_idx: use cached position + forward scan (O(1) typical).
            // On first call after restart, cached is 0 so fall back to binary search.
            let start_idx = if unlikely(cached[boundary_idx] == 0 && chain_state.len() > 1) {
                let idx = chain_state.partition_point(|b| *b.timestamp <= lower_timestamp);
                cached[boundary_idx] = idx;
                idx
            } else {
                let mut idx = cached[boundary_idx];
                while idx < chain_state.len() && *chain_state[idx].timestamp <= lower_timestamp {
                    idx += 1;
                }
                cached[boundary_idx] = idx;
                idx
            };

            // Linear scan for end (typically 0-2 blocks past start)
            let end_idx = chain_state[start_idx..]
                .iter()
                .position(|b| *b.timestamp > upper_timestamp)
                .map_or(chain_state.len(), |pos| start_idx + pos);

            // Move supply from younger cohort to older cohort
            let younger = adjacent[0];
            let older = adjacent[1];
            let matured_sats = younger.select_mut(&mut matured);
            for block_state in &chain_state[start_idx..end_idx] {
                let snapshot = CostBasisSnapshot::from_utxo(block_state.price, &block_state.supply);
                younger.select_mut(age_range).decrement_snapshot(&snapshot);
                older.select_mut(age_range).increment_snapshot(&snapshot);
                *matured_sats += block_state.supply.value;
            }
        }

        matured
    }
}
