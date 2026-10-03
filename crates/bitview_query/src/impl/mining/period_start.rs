use bitview_types::TimePeriod;
use brk_types::Height;
use vecdb::ReadableVec;

use crate::{Error as QueryError, OptionData, Query, Result};

/// First block height inside `period` looking back from the tip;
/// genesis (`Height(0)`) for `All`. Errors `Internal` if the chosen
/// lookback vec is stamped short of the tip - separating the
/// "all-time" case from a transient stamp-lag dropout that would
/// otherwise silently widen a windowed query to the full chain.
pub fn start_height(query: &Query, period: TimePeriod) -> Result<Height> {
    start_height_at(query, period, query.height())
}

pub fn start_height_at(query: &Query, period: TimePeriod, tip: Height) -> Result<Height> {
    let lookback = &query.plugins().blocks.lookback;
    let start = match period {
        TimePeriod::Day => lookback._24h.collect_one(tip).data()?,
        TimePeriod::ThreeDays => lookback._3d.collect_one(tip).data()?,
        TimePeriod::Week => lookback._1w.collect_one(tip).data()?,
        TimePeriod::Month => lookback._1m.collect_one(tip).data()?,
        TimePeriod::ThreeMonths => lookback._3m.collect_one(tip).data()?,
        TimePeriod::SixMonths => lookback._6m.collect_one(tip).data()?,
        TimePeriod::Year => lookback._1y.collect_one(tip).data()?,
        TimePeriod::TwoYears => lookback._2y.collect_one(tip).data()?,
        TimePeriod::ThreeYears => lookback._3y.collect_one(tip).data()?,
        TimePeriod::All => Height::from(0_usize),
    };
    if start > tip {
        return Err(QueryError::Internal(
            "Mining lookback exceeds published tip",
        ));
    }
    Ok(start)
}
