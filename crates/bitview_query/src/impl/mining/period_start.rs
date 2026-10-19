use bitview_types::TimePeriod;
use brk_types::Height;
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, Result};

/// The first height of `period`'s timestamp lookback ending at `tip`; genesis for `All`.
/// `Internal` when the lookback is missing or past `tip`.
pub fn start_height(query: &Query, period: TimePeriod, tip: Height) -> Result<Height> {
    let lookback = &query.plugins().mappings.lookback;
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
        return Err(Error::Internal("Mining lookback exceeds published tip"));
    }
    Ok(start)
}
