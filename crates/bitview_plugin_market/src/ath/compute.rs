use bitview_compute::ComputeRollingStats;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{Days, Seconds};
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
        let starting_height = indexer.compute_start().height;

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

        compute_max_days_between(
            &mut self.max_days_between.height,
            starting_height,
            &self.seconds_since.height,
            &mappings.timestamp.monotonic,
            exit,
        )?;

        Ok(())
    }
}

fn compute_seconds_since(
    seconds_since: &mut CachedSeries<Height, Seconds>,
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
                (i, Seconds::ZERO)
            } else {
                (i, Seconds::from(*ts - *ath_ts.unwrap()))
            }
        },
        exit,
    )?;

    Ok(())
}

/// Running maximum of the time elapsed since the previous ATH, measured before an ATH resets
/// it: a completed interval counts at the block that ends it.
fn compute_max_days_between(
    max_days_between: &mut CachedSeries<Height, Days>,
    starting_height: Height,
    seconds_since: &impl ReadableVec<Height, Seconds>,
    timestamps: &impl ReadableVec<Height, Timestamp>,
    exit: &Exit,
) -> Result<()> {
    // The previous block's running maximum, seconds since its ATH and timestamp.
    let mut prev: Option<(Days, Seconds, Timestamp)> = None;
    max_days_between.compute_transform2(
        starting_height,
        seconds_since,
        timestamps,
        |(i, seconds, ts, slf)| {
            if prev.is_none()
                && let Some(h) = i.decremented()
            {
                prev = Some((
                    slf.collect_one(h).unwrap(),
                    seconds_since.collect_one(h).unwrap(),
                    timestamps.collect_one(h).unwrap(),
                ));
            }
            let max = prev.map_or(Days::default(), |(max, prev_seconds, prev_ts)| {
                let elapsed = Seconds::from(*prev_seconds + (*ts - *prev_ts));
                max.max(SecondsToDays::apply(elapsed))
            });
            prev = Some((max, seconds, ts));
            (i, max)
        },
        exit,
    )?;
    Ok(())
}
