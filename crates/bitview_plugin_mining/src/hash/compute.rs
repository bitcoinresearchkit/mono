use bitview_compute::ComputeRollingStats;
use bitview_plugin_blocks::DifficultyVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_primitives::{HashPrice, HashValue, Hashrate, PartsPerMillionSigned32};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height, Sats, TARGET_BLOCKS_PER_DAY_F64};
use vecdb::{AnyVec, ComputeVec, LazyVec, ReadableCloneableVec, ReadableVec};

use super::Vecs;

const ONE_PETA_HASH: f64 = 1_000_000_000_000_000.0;

/// Daily reward per PH/s. Every block proves positive work, so the hash rate is positive.
#[inline]
fn reward_per_phs(reward_24h: f64, hash_rate: f64) -> f64 {
    reward_24h / (hash_rate / ONE_PETA_HASH)
}

impl Vecs {
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
        let daily_share = LazyVec::new(
            "difficulty_hashrate_daily_share",
            difficulty_vecs.hashrate.height.version(),
            difficulty_vecs.hashrate.height.read_only_boxed_clone(),
            |_, hashrate: Hashrate| Hashrate::from(f64::from(hashrate) / TARGET_BLOCKS_PER_DAY_F64),
        );
        self.hashrate.value.height.compute_rolling_sum(
            starting_height,
            lookback.start_vec(1),
            &daily_share,
            exit,
        )?;

        let hash_rate = &self.hashrate.value.height;
        let sma = &mut self.hashrate.sma;
        let [week, month, two_months, year] = &mut sma.windows;
        for (sma, window, retained) in [
            (&mut sma._1w.height, lookback.start_vec(7), week),
            (&mut sma._1m.height, lookback.start_vec(30), month),
            (&mut sma._2m.height, lookback.start_vec(60), two_months),
            (&mut sma._1y.height, lookback.start_vec(365), year),
        ] {
            sma.compute_rolling_average(starting_height, window, hash_rate, retained, exit)?;
        }

        self.hashrate.ath.height.compute_all_time_high(
            starting_height,
            &self.hashrate.value.height,
            exit,
        )?;

        self.hashrate.drawdown.fixed.height.compute_transform2(
            starting_height,
            &self.hashrate.value.height,
            &self.hashrate.ath.height,
            |(_, current, ath, ..)| {
                let ath = f64::from(ath);
                PartsPerMillionSigned32::from((f64::from(current) - ath) / ath)
            },
            exit,
        )?;

        // (NaN until the 24-hour window is fully priced, as its dollar sum: a partly priced sum would
        // set a false low.)
        self.hash_price.value.height.compute_transform2(
            starting_height,
            coinbase_usd_24h_sum,
            &self.hashrate.value.height,
            |(_, coinbase_sum, hashrate, ..)| {
                HashPrice::from(reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate)))
            },
            exit,
        )?;

        self.hash_value.value.height.compute_transform2(
            starting_height,
            coinbase_sats_24h_sum,
            &self.hashrate.value.height,
            |(_, coinbase_sum, hashrate, ..)| {
                HashValue::from(reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate)))
            },
            exit,
        )?;

        self.hash_price
            .compute_floor_and_rebound(starting_height, exit)?;
        self.hash_value.compute_floor_and_rebound(starting_height, exit)
    }
}
