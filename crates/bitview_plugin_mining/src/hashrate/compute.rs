use bitview_compute::ComputeRollingStats;
use bitview_plugin_blocks::DifficultyVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::FIRST_PRICE_HEIGHT;
use bitview_primitives::{HashPrice, HashValue, Hashrate, PartsPerMillionSigned32};
use bitview_vecs::ops::RelativeChange;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height, Sats, TARGET_BLOCKS_PER_DAY_F64};
use vecdb::{AnyVec, ComputeVec, LazyVec, ReadableCloneableVec, ReadableVec};

use super::Vecs;

const ONE_PETA_HASH: f64 = 1_000_000_000_000_000.0;

/// Daily reward per PH/s; NaN without hash rate.
#[inline]
fn reward_per_phs(reward_24h: f64, hash_rate: f64) -> f64 {
    let hash_rate_phs = hash_rate / ONE_PETA_HASH;
    if hash_rate_phs == 0.0 {
        f64::NAN
    } else {
        reward_24h / hash_rate_phs
    }
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        difficulty_vecs: &DifficultyVecs,
        coinbase_sats_24h_sum: &impl ReadableVec<Height, Sats>,
        coinbase_usd_24h_sum: &impl ReadableVec<Height, Dollars>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;

        // Each block's share of a day's work at the ten-minute target, summed over 24 hours.
        let daily_share = LazyVec::init(
            "difficulty_hashrate_daily_share",
            difficulty_vecs.hashrate.height.version(),
            difficulty_vecs.hashrate.height.read_only_boxed_clone(),
            |_, hashrate: Hashrate| Hashrate::from(f64::from(hashrate) / TARGET_BLOCKS_PER_DAY_F64),
        );
        self.rate.block.height.compute_rolling_sum(
            starting_height,
            lookback.start_vec(1),
            &daily_share,
            exit,
        )?;

        let hash_rate = &self.rate.block.height;
        let sma = &mut self.rate.sma;
        let [week, month, two_months, year] = &mut sma.windows;
        for (sma, window, retained) in [
            (&mut sma._1w.height, lookback.start_vec(7), week),
            (&mut sma._1m.height, lookback.start_vec(30), month),
            (&mut sma._2m.height, lookback.start_vec(60), two_months),
            (&mut sma._1y.height, lookback.start_vec(365), year),
        ] {
            sma.compute_rolling_average(starting_height, window, hash_rate, retained, exit)?;
        }

        self.rate.ath.height.compute_all_time_high(
            starting_height,
            &self.rate.block.height,
            exit,
        )?;

        self.rate.drawdown.fixed.height.compute_transform2(
            starting_height,
            &self.rate.block.height,
            &self.rate.ath.height,
            |(i, current, ath, ..)| {
                let ath = f64::from(ath);
                let drawdown = if ath == 0.0 {
                    PartsPerMillionSigned32::default()
                } else {
                    PartsPerMillionSigned32::from((f64::from(current) - ath) / ath)
                };
                (i, drawdown)
            },
            exit,
        )?;

        // NaN until the 24-hour window is fully priced: a partly priced sum would set a false low.
        let first_priced = Height::from(FIRST_PRICE_HEIGHT);
        self.price.block.height.compute_transform3(
            starting_height,
            coinbase_usd_24h_sum,
            &self.rate.block.height,
            lookback.start_vec(1),
            |(i, coinbase_sum, hashrate, start, ..)| {
                let price = if start < first_priced {
                    HashPrice::NAN
                } else {
                    HashPrice::from(reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate)))
                };
                (i, price)
            },
            exit,
        )?;

        self.value.block.height.compute_transform2(
            starting_height,
            coinbase_sats_24h_sum,
            &self.rate.block.height,
            |(i, coinbase_sum, hashrate, ..)| {
                (
                    i,
                    HashValue::from(reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate))),
                )
            },
            exit,
        )?;

        self.price.atl.height.compute_all_time_low(
            starting_height,
            &self.price.block.height,
            exit,
        )?;
        self.value.atl.height.compute_all_time_low(
            starting_height,
            &self.value.block.height,
            exit,
        )?;

        self.price
            .rebound
            .fixed
            .height
            .compute_binary::<HashPrice, HashPrice, RelativeChange<PartsPerMillionSigned32>>(
                starting_height,
                &self.price.block.height,
                &self.price.atl.height,
                exit,
            )?;

        self.value
            .rebound
            .fixed
            .height
            .compute_binary::<HashValue, HashValue, RelativeChange<PartsPerMillionSigned32>>(
                starting_height,
                &self.value.block.height,
                &self.value.atl.height,
                exit,
            )?;

        Ok(())
    }
}
