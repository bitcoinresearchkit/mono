use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_types::{CheckedSub, Halving, Sats};
use vecdb::{ReadableVec, VecIndex};

use super::Vecs;

fn derived_subsidy(height: brk_types::Height, coinbase: Sats, fees: Sats) -> Sats {
    coinbase
        .checked_sub(fees)
        .unwrap_or_else(|| panic!("coinbase {coinbase:?} < fees {fees:?} at {height:?}"))
}

fn scheduled_subsidy(height: brk_types::Height) -> Sats {
    let halving = Halving::from(height);
    Sats::FIFTY_BTC / 2_usize.pow(halving.to_usize() as u32)
}

fn unclaimed_rewards(height: brk_types::Height, subsidy: Sats) -> Sats {
    scheduled_subsidy(height)
        .checked_sub(subsidy)
        .unwrap_or_else(|| panic!("derived subsidy {subsidy:?} exceeds schedule at {height:?}"))
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    mappings: &bitview_plugin_mappings::Vecs,
    lookback: &bitview_plugin_blocks::LookbackVecs,
    transactions: &bitview_plugin_transactions::Vecs,
    prices: &bitview_plugin_price::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, mappings, lookback, transactions, prices, exit)
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        lookback: &bitview_plugin_blocks::LookbackVecs,
        transactions: &bitview_plugin_transactions::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        // coinbase and fees are independent — parallelize
        let window_starts = lookback.window_starts();
        let (r_coinbase, r_fees) = rayon::join(
            || {
                self.coinbase.compute_from(
                    starting_height,
                    &prices.spot.cents.height,
                    &indexer.vecs().transactions.first_tx_index,
                    |_, tx_index| {
                        let mut txout_cursor = indexer
                            .vecs()
                            .transactions
                            .first_txout_index
                            .reader()
                            .cursor();
                        let mut count_cursor = mappings.tx_index.output_count.cursor();

                        let ti = tx_index.to_usize();

                        txout_cursor.advance(ti - txout_cursor.position());
                        let first_txout_index = txout_cursor.next().unwrap().to_usize();

                        count_cursor.advance(ti - count_cursor.position());
                        let output_count: usize = count_cursor.next().unwrap().into();

                        indexer.vecs().outputs.value.fold_range_at(
                            first_txout_index,
                            first_txout_index + output_count,
                            Sats::ZERO,
                            |acc, v| acc + v,
                        )
                    },
                    exit,
                )
            },
            || {
                self.fees.compute_from_indexes(
                    starting_height,
                    &window_starts,
                    &prices.spot.cents.height,
                    &indexer.vecs().transactions.first_tx_index,
                    &mappings.height.tx_index_count,
                    &transactions.fees.fee.tx_index,
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

#[cfg(test)]
mod tests {
    use brk_types::{Height, Sats};

    use super::{derived_subsidy, scheduled_subsidy, unclaimed_rewards};

    #[test]
    fn reward_components_match_the_available_reward_equation() {
        let height = Height::from(0_u32);
        let fees = Sats::ONE_BTC;

        let fully_claimed_coinbase = Sats::FIFTY_BTC + fees;
        let subsidy = derived_subsidy(height, fully_claimed_coinbase, fees);
        assert_eq!(subsidy, Sats::FIFTY_BTC);
        assert_eq!(unclaimed_rewards(height, subsidy), Sats::ZERO);

        let underclaimed_coinbase = Sats::FIFTY_BTC;
        let subsidy = derived_subsidy(height, underclaimed_coinbase, fees);
        assert_eq!(subsidy, Sats::FIFTY_BTC - fees);
        assert_eq!(unclaimed_rewards(height, subsidy), fees);
    }

    #[test]
    fn scheduled_subsidy_halves_at_210_000_blocks() {
        assert_eq!(
            scheduled_subsidy(Height::from(209_999_u32)),
            Sats::FIFTY_BTC
        );
        assert_eq!(
            scheduled_subsidy(Height::from(210_000_u32)),
            Sats::FIFTY_BTC / 2
        );
    }
}
