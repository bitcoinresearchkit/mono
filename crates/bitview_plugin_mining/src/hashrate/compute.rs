use bitview_compute::ComputeRollingStats;
use bitview_plugin_blocks::{CountVecs, DifficultyVecs, LookbackVecs, ONE_TERA_HASH};
use bitview_plugin_indexer::Indexer;
use bitview_transforms::RatioDiffF32;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{
    Dollars, Height, PartsPerMillionSigned32, Sats, StoredF32, StoredF64, TARGET_BLOCKS_PER_DAY_F64,
};
use vecdb::ReadableVec;

use super::Vecs;

#[inline]
fn estimated_network_hash_rate(block_count_24h: f64, difficulty_hash_rate: f64) -> f64 {
    (block_count_24h / TARGET_BLOCKS_PER_DAY_F64) * difficulty_hash_rate
}

#[inline]
fn reward_per_ths(reward_24h: f64, hash_rate: f64) -> StoredF32 {
    let hash_rate_ths = hash_rate / ONE_TERA_HASH;
    if hash_rate_ths == 0.0 {
        StoredF32::NAN
    } else {
        StoredF32::from(reward_24h / hash_rate_ths)
    }
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        count_vecs: &CountVecs,
        lookback: &LookbackVecs,
        difficulty_vecs: &DifficultyVecs,
        coinbase_sats_24h_sum: &impl ReadableVec<Height, Sats>,
        coinbase_usd_24h_sum: &impl ReadableVec<Height, Dollars>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.rate.base.height.compute_transform2(
            starting_height,
            &count_vecs.total.sum._24h.height,
            &difficulty_vecs.hashrate.height,
            |(i, block_count_sum, difficulty_as_hash, ..)| {
                (
                    i,
                    StoredF64::from(estimated_network_hash_rate(
                        f64::from(block_count_sum),
                        f64::from(difficulty_as_hash),
                    )),
                )
            },
            exit,
        )?;

        let hash_rate = &self.rate.base.height;
        for (sma, window) in [
            (&mut self.rate.sma._1w.height, lookback.start_vec(7)),
            (&mut self.rate.sma._1m.height, lookback.start_vec(30)),
            (&mut self.rate.sma._2m.height, lookback.start_vec(60)),
            (&mut self.rate.sma._1y.height, lookback.start_vec(365)),
        ] {
            sma.compute_rolling_average(starting_height, window, hash_rate, exit)?;
        }

        self.rate.ath.height.compute_all_time_high(
            starting_height,
            &self.rate.base.height,
            exit,
        )?;

        self.rate.drawdown.compute_drawdown(
            starting_height,
            &self.rate.base.height,
            &self.rate.ath.height,
            exit,
        )?;

        self.price.ths.height.compute_transform2(
            starting_height,
            coinbase_usd_24h_sum,
            &self.rate.base.height,
            |(i, coinbase_sum, hashrate, ..)| {
                (i, reward_per_ths(f64::from(coinbase_sum), *hashrate))
            },
            exit,
        )?;

        self.value.ths.height.compute_transform2(
            starting_height,
            coinbase_sats_24h_sum,
            &self.rate.base.height,
            |(i, coinbase_sum, hashrate, ..)| {
                (i, reward_per_ths(f64::from(coinbase_sum), *hashrate))
            },
            exit,
        )?;

        for (min_vec, src_vec) in [
            (&mut self.price.ths_min.height, &self.price.ths.height),
            (&mut self.value.ths_min.height, &self.value.ths.height),
        ] {
            min_vec.compute_all_time_low(starting_height, src_vec, exit, true)?;
        }

        self.price
            .rebound
            .compute_binary::<StoredF32, StoredF32, RatioDiffF32<PartsPerMillionSigned32>>(
                starting_height,
                &self.price.phs.height,
                &self.price.phs_min.height,
                exit,
            )?;

        self.value
            .rebound
            .compute_binary::<StoredF32, StoredF32, RatioDiffF32<PartsPerMillionSigned32>>(
                starting_height,
                &self.value.phs.height,
                &self.value.phs_min.height,
                exit,
            )?;

        Ok(())
    }
}
