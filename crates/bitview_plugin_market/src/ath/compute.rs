use brk_error::Result;

use brk_indexer::Indexer;
use brk_types::{StoredF32, Timestamp};
use vecdb::{Exit, ReadableVec, VecIndex};

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &bitview_plugin_price::Vecs,
    indexes: &bitview_plugin_indexes::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, prices, indexes, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        indexes: &bitview_plugin_indexes::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.high.cents.height.compute_all_time_high(
            starting_height,
            &prices.spot.cents.height,
            exit,
        )?;

        let mut ath_ts: Option<Timestamp> = None;
        self.days_since.height.compute_transform3(
            starting_height,
            &self.high.cents.height,
            &prices.spot.cents.height,
            &indexes.timestamp.monotonic,
            |(i, ath, price, ts, slf)| {
                if ath_ts.is_none() {
                    let idx = i.to_usize();
                    ath_ts = Some(if idx > 0 {
                        let prev_days: StoredF32 = slf.collect_one_at(idx - 1).unwrap();
                        Timestamp::from((*ts as f64 - *prev_days as f64 * 86400.0) as u32)
                    } else {
                        ts
                    });
                }
                if price == ath {
                    ath_ts = Some(ts);
                    (i, StoredF32::default())
                } else {
                    let days = ts.difference_in_days_between_float(ath_ts.unwrap());
                    (i, StoredF32::from(days as f32))
                }
            },
            exit,
        )?;

        let mut prev = None;
        self.max_days_between.height.compute_transform(
            starting_height,
            &self.days_since.height,
            |(i, days, slf)| {
                if prev.is_none() {
                    let i = i.to_usize();
                    prev.replace(if i > 0 {
                        slf.collect_one_at(i - 1).unwrap()
                    } else {
                        StoredF32::default()
                    });
                }
                let max = prev.unwrap().max(days);
                prev.replace(max);
                (i, max)
            },
            exit,
        )?;

        Ok(())
    }
}
