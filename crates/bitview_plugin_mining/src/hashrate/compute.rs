use bitview_compute::ComputeRollingStats;
use bitview_plugin_blocks::{DifficultyVecs, LookbackVecs};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Count, Float32, Hashrate, PartsPerMillionSigned32};
use bitview_transforms::RatioDiffFloat32;
use bitview_vecs::LazyPerBlockCumulativeRolling;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height, Sats, TARGET_BLOCKS_PER_DAY_F64};
use vecdb::ReadableVec;

use super::Vecs;

#[inline]
fn estimated_network_hash_rate(block_count_24h: f64, difficulty_hash_rate: f64) -> f64 {
    (block_count_24h / TARGET_BLOCKS_PER_DAY_F64) * difficulty_hash_rate
}

const ONE_PETA_HASH: f64 = 1_000_000_000_000_000.0;

#[inline]
fn reward_per_phs(reward_24h: f64, hash_rate: f64) -> Float32 {
    let hash_rate_phs = hash_rate / ONE_PETA_HASH;
    if hash_rate_phs == 0.0 {
        Float32::NAN
    } else {
        Float32::from(reward_24h / hash_rate_phs)
    }
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        block_count: &LazyPerBlockCumulativeRolling<Count>,
        lookback: &LookbackVecs,
        difficulty_vecs: &DifficultyVecs,
        coinbase_sats_24h_sum: &impl ReadableVec<Height, Sats>,
        coinbase_usd_24h_sum: &impl ReadableVec<Height, Dollars>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.rate.block.height.compute_transform2(
            starting_height,
            &block_count.sum._24h.height,
            &difficulty_vecs.hashrate.height,
            |(i, block_count_sum, difficulty_as_hash, ..)| {
                (
                    i,
                    Hashrate::from(estimated_network_hash_rate(
                        f64::from(block_count_sum),
                        f64::from(difficulty_as_hash),
                    )),
                )
            },
            exit,
        )?;

        let hash_rate = &self.rate.block.height;
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
            &self.rate.block.height,
            exit,
        )?;

        self.rate.drawdown.compute_drawdown(
            starting_height,
            &self.rate.block.height,
            &self.rate.ath.height,
            exit,
        )?;

        self.price.block.height.compute_transform2(
            starting_height,
            coinbase_usd_24h_sum,
            &self.rate.block.height,
            |(i, coinbase_sum, hashrate, ..)| {
                (
                    i,
                    reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate)),
                )
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
                    reward_per_phs(f64::from(coinbase_sum), f64::from(hashrate)),
                )
            },
            exit,
        )?;

        for (min_vec, src_vec) in [
            (&mut self.price.atl.height, &self.price.block.height),
            (&mut self.value.atl.height, &self.value.block.height),
        ] {
            min_vec.compute_all_time_low(starting_height, src_vec, exit, true)?;
        }

        self.price
            .rebound
            .compute_binary::<Float32, Float32, RatioDiffFloat32<PartsPerMillionSigned32>>(
                starting_height,
                &self.price.block.height,
                &self.price.atl.height,
                exit,
            )?;

        self.value
            .rebound
            .compute_binary::<Float32, Float32, RatioDiffFloat32<PartsPerMillionSigned32>>(
                starting_height,
                &self.value.block.height,
                &self.value.atl.height,
                exit,
            )?;

        Ok(())
    }
}
