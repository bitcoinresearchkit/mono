use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::Halving;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{CheckedSub, Height, Sats};
use rayon::join;

use super::Vecs;

/// Zero when the coinbase claims less than the fees: the unclaimed fees are burned.
fn claimed_subsidy(_: Height, coinbase: Sats, fees: Sats) -> Sats {
    coinbase.checked_sub(fees).unwrap_or(Sats::ZERO)
}

/// Consensus caps the coinbase at the scheduled subsidy plus fees.
fn unclaimed_rewards(height: Height, coinbase: Sats, fees: Sats) -> Sats {
    Halving::from(height).subsidy() + fees - coinbase
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        transactions: &TransactionsVecs,
        prices: &PriceVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;

        // coinbase and fees are independent — parallelize
        let window_starts = lookback.window_starts();
        let (r_coinbase, r_fees) = join(
            || {
                self.coinbase.compute_from(
                    starting_height,
                    &prices.spot.cents.height,
                    &transactions.fees.coinbase_value,
                    |_, value| value,
                    exit,
                )
            },
            || {
                self.fees.compute_from(
                    starting_height,
                    &window_starts,
                    &prices.spot.cents.height,
                    &transactions.fees.total,
                    exit,
                )
            },
        );
        r_coinbase?;
        r_fees?;

        self.subsidy.compute_from_pair(
            starting_height,
            &prices.spot.cents.height,
            &self.coinbase.block.sats,
            &self.fees.block.sats,
            claimed_subsidy,
            exit,
        )?;

        self.unclaimed.compute_from_pair(
            starting_height,
            &prices.spot.cents.height,
            &self.coinbase.block.sats,
            &self.fees.block.sats,
            unclaimed_rewards,
            exit,
        )?;

        Ok(())
    }
}
