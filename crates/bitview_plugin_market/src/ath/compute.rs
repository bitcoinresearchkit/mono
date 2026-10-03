use bitview_compute::ComputeRollingStats;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{StoredF32, StoredU32};
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Timestamp};
use vecdb::{ReadableVec, UnaryTransform, VecIndex};

use super::{Vecs, seconds_to_days::SecondsToDays};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.high.cents.height.compute_all_time_high(
            starting_height,
            &prices.spot.cents.height,
            exit,
        )?;

        compute_seconds_since(
            &mut self.seconds_since.height,
            starting_height,
            &self.high.cents.height,
            &prices.spot.cents.height,
            &mappings.timestamp.monotonic,
            exit,
        )?;

        let mut prev = None;
        self.max_days_between.height.compute_transform(
            starting_height,
            &self.seconds_since.height,
            |(i, seconds, slf)| {
                if prev.is_none() {
                    let i = i.to_usize();
                    prev.replace(if i > 0 {
                        slf.collect_one_at(i - 1).unwrap()
                    } else {
                        StoredF32::default()
                    });
                }
                let max = prev.unwrap().max(SecondsToDays::apply(seconds));
                prev.replace(max);
                (i, max)
            },
            exit,
        )?;

        Ok(())
    }
}

fn compute_seconds_since(
    seconds_since: &mut CachedSeries<Height, StoredU32>,
    starting_height: Height,
    high: &impl ReadableVec<Height, Cents>,
    prices: &impl ReadableVec<Height, Cents>,
    timestamps: &impl ReadableVec<Height, Timestamp>,
    exit: &Exit,
) -> Result<()> {
    let mut ath_ts: Option<Timestamp> = None;
    seconds_since.compute_transform3(
        starting_height,
        high,
        prices,
        timestamps,
        |(i, ath, price, ts, slf)| {
            if ath_ts.is_none() {
                let idx = i.to_usize();
                ath_ts = Some(if idx > 0 {
                    let prev_seconds = slf.collect_one_at(idx - 1).unwrap();
                    let prev_ts = timestamps.collect_one_at(idx - 1).unwrap();
                    Timestamp::from(*prev_ts - *prev_seconds)
                } else {
                    ts
                });
            }
            if price == ath {
                ath_ts = Some(ts);
                (i, StoredU32::ZERO)
            } else {
                (i, StoredU32::from(*ts - *ath_ts.unwrap()))
            }
        },
        exit,
    )?;

    Ok(())
}
