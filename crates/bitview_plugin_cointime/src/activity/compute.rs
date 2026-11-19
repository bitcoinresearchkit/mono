use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{BoundedRatio, CoinBlocks};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, Version};
use vecdb::{AnyVec, ComputeVec, ReadableVec, WritableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        age: &AgeVecs,
        supply: &impl ReadableVec<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;

        let created = &mut self.coinblocks_created;
        let version = created.block.version() + Version::ONE;
        let last = &mut self.created_sat_blocks;
        created.cumulative.height.compute_to(
            starting_height,
            supply.len(),
            version,
            |target, range| {
                let mut height = range.start;
                supply.for_each_range_at(range.start, range.end, |sats| {
                    let total = last.accumulate(
                        height,
                        || Some(supply.fold_range_at(0, height, 0, add_sats)),
                        |total| *total = add_sats(*total, sats),
                    );
                    target.push(coinblocks(total));
                    height += 1;
                });
                Ok(())
            },
            exit,
        )?;

        self.coinblocks_stored.cumulative.height.compute_subtract(
            starting_height,
            &self.coinblocks_created.cumulative.height,
            &age.coinblocks_destroyed.cumulative.height,
            exit,
        )?;

        self.derived.liveliness_source.height.compute_transform2(
            starting_height,
            &age.coinblocks_destroyed.cumulative.height,
            &self.coinblocks_created.cumulative.height,
            |(_, destroyed, created, ..)| {
                BoundedRatio::from(f64::from(destroyed) / f64::from(created))
            },
            exit,
        )?;

        Ok(())
    }
}

fn add_sats(total: u128, sats: Sats) -> u128 {
    total + u128::from(u64::from(sats))
}

fn coinblocks(sat_blocks: u128) -> CoinBlocks {
    if sat_blocks == 0 {
        return CoinBlocks::default();
    }
    const BTC: u128 = Sats::ONE_BTC_U128;
    const PRECISION: u32 = f64::MANTISSA_DIGITS;
    // Scale for a 53-bit significand, then round the integer division once, ties to even.
    let mut shift = (PRECISION + BTC.ilog2()) as i32 - sat_blocks.ilog2() as i32;
    let (numerator, mut divisor) = if shift >= 0 {
        (sat_blocks << shift, BTC)
    } else {
        (sat_blocks, BTC << -shift)
    };
    if numerator >= divisor << PRECISION {
        divisor *= 2;
        shift -= 1;
    }
    let significand = numerator / divisor;
    let twice_remainder = 2 * (numerator % divisor);
    let round_up =
        twice_remainder > divisor || (twice_remainder == divisor && significand % 2 == 1);
    CoinBlocks::new((significand + u128::from(round_up)) as f64 * 2.0_f64.powi(-shift))
}
