use bitview_plugin_blocks::LookbackVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::Halving;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{CheckedSub, Height, Sats};
use rayon::join;
use vecdb::VecIndex;

use super::Vecs;

fn derived_subsidy(height: Height, coinbase: Sats, fees: Sats) -> Sats {
    coinbase
        .checked_sub(fees)
        .unwrap_or_else(|| panic!("coinbase {coinbase:?} < fees {fees:?} at {height:?}"))
}

fn scheduled_subsidy(height: Height) -> Sats {
    let halving = Halving::from(height);
    Sats::FIFTY_BTC / 2_usize.pow(halving.to_usize() as u32)
}

fn unclaimed_rewards(height: Height, subsidy: Sats) -> Sats {
    scheduled_subsidy(height)
        .checked_sub(subsidy)
        .unwrap_or_else(|| panic!("derived subsidy {subsidy:?} exceeds schedule at {height:?}"))
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
        let starting_height = indexer.safe_lengths().height;

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
            derived_subsidy,
            exit,
        )?;

        self.output_volume.compute_subtract(
            starting_height,
            &transactions.volume.transfer_volume.block.sats,
            &self.fees.block.sats,
            exit,
        )?;

        self.unclaimed.compute_from(
            starting_height,
            &prices.spot.cents.height,
            &self.subsidy.block.sats,
            unclaimed_rewards,
            exit,
        )?;

        Ok(())
    }
}
