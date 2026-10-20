use bitview_compute::ComputeRollingStats;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{Days, Seconds};
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Timestamp};
use vecdb::{ComputeVec, ReadableVec, UnaryTransform};

use super::{
    Vecs,
    seconds_to_days::{SecondsToDays, UNPRICED},
};

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

        compute_days_between_max(
            &mut self.days_between_max.height,
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
    // The latest ATH's timestamp, from the row before the first one computed; `None` before the
    // first price, whose zero ATH is none.
    let mut ath_ts: Option<Option<Timestamp>> = None;
    seconds_since.compute_transform3(
        starting_height,
        high,
        prices,
        timestamps,
        |(i, ath, price, ts, slf)| {
            let ath_ts = ath_ts.get_or_insert_with(|| {
                let prev = i.decremented()?;
                let seconds = slf.collect_one(prev).unwrap();
                (seconds != UNPRICED)
                    .then(|| Timestamp::from(*timestamps.collect_one(prev).unwrap() - *seconds))
            });
            if ath == Cents::ZERO {
                UNPRICED
            } else if price == ath {
                *ath_ts = Some(ts);
                Seconds::ZERO
            } else {
                Seconds::from(*ts - *ath_ts.unwrap())
            }
        },
        exit,
    )?;

    Ok(())
}

/// Running maximum of the time elapsed since the previous ATH, measured before an ATH resets
/// it: a completed interval counts at the block that ends it. Null before the first price, the
/// first ATH.
fn compute_days_between_max(
    days_between_max: &mut CachedSeries<Height, Days>,
    starting_height: Height,
    seconds_since: &impl ReadableVec<Height, Seconds>,
    timestamps: &impl ReadableVec<Height, Timestamp>,
    exit: &Exit,
) -> Result<()> {
    // The previous block's running maximum, seconds since its ATH and timestamp; `None` before
    // the first price.
    let mut prev: Option<Option<(Days, Seconds, Timestamp)>> = None;
    days_between_max.compute_transform2(
        starting_height,
        seconds_since,
        timestamps,
        |(i, seconds, ts, slf)| {
            let prev = prev.get_or_insert_with(|| {
                let h = i.decremented()?;
                let seconds = seconds_since.collect_one(h).unwrap();
                (seconds != UNPRICED).then(|| {
                    (
                        slf.collect_one(h).unwrap(),
                        seconds,
                        timestamps.collect_one(h).unwrap(),
                    )
                })
            });
            if seconds == UNPRICED {
                return Days::NAN;
            }
            let max = match *prev {
                Some((max, prev_seconds, prev_ts)) => {
                    let elapsed = Seconds::from(*prev_seconds + (*ts - *prev_ts));
                    max.max(SecondsToDays::apply(elapsed))
                }
                // The first price is the first ATH.
                None => SecondsToDays::apply(seconds),
            };
            *prev = Some((max, seconds, ts));
            max
        },
        exit,
    )?;
    Ok(())
}
