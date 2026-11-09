use bitview_plugin_mappings::LookbackVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CheckedSub, Height, Sats};
use rayon::join;
use vecdb::ReadableVec;

use super::Vecs;

/// Zero when the coinbase claims less than the fees: the unclaimed fees are burned.
fn claimed_subsidy(_: Height, coinbase: Sats, fees: Sats) -> Sats {
    coinbase.checked_sub(fees).unwrap_or(Sats::ZERO)
}

/// Consensus caps the coinbase at the scheduled subsidy plus fees.
fn unclaimed_rewards(height: Height, coinbase: Sats, fees: Sats) -> Sats {
    height.subsidy() + fees - coinbase
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        lookback: &LookbackVecs,
        coinbase_value: &impl ReadableVec<Height, Sats>,
        total_fees: &impl ReadableVec<Height, Sats>,
        spot_cents: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        // coinbase and fees are independent — parallelize
        let window_starts = lookback.window_starts();
        let (r_coinbase, r_fees) = join(
            || {
                self.coinbase.compute_from(
                    starting_height,
                    spot_cents,
                    coinbase_value,
                    |_, value| value,
                    exit,
                )
            },
            || {
                self.fees.compute_from(
                    starting_height,
                    &window_starts,
                    spot_cents,
                    total_fees,
                    exit,
                )
            },
        );
        r_coinbase?;
        r_fees?;

        self.subsidy.compute_from_pair(
            starting_height,
            spot_cents,
            &self.coinbase.block.sats,
            &self.fees.block.sats,
            claimed_subsidy,
            exit,
        )?;

        self.unclaimed.compute_from_pair(
            starting_height,
            spot_cents,
            &self.coinbase.block.sats,
            &self.fees.block.sats,
            unclaimed_rewards,
            exit,
        )?;

        Ok(())
    }
}
