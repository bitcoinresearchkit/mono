use bitview_primitives::{
    Count, Date, Day1, Day3, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, Week1, Year1, Year10,
};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPreviousDeltaVec;
use brk_types::{Height, Timestamp, Version};
use vecdb::{IndexVec, LazyVec, ReadableBoxedVec, ReadableVec, VecValue};

use crate::RangeMapLookupVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based 10-minute UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub minute10: LazyVec<Height, Minute10, Height, Timestamp>,
    /// Zero-based 30-minute UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub minute30: LazyVec<Height, Minute30, Height, Timestamp>,
    /// Zero-based one-hour UTC period containing the block's monotonic timestamp,
    /// counted from 2009-01-01 00:00:00 UTC.
    pub hour1: LazyVec<Height, Hour1, Height, Timestamp>,
    /// Zero-based four-hour UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub hour4: LazyVec<Height, Hour4, Height, Timestamp>,
    /// Zero-based 12-hour UTC period containing the block's monotonic timestamp,
    /// counted from 2009-01-01 00:00:00 UTC.
    pub hour12: LazyVec<Height, Hour12, Height, Timestamp>,
    /// Zero-based UTC calendar day containing the block's monotonic timestamp,
    /// with 2009-01-01 equal to 0.
    pub day1: RangeMapLookupVec<Height, Day1>,
    /// Zero-based three-day UTC period containing the block's monotonic
    /// timestamp, with period 1 beginning on 2009-01-03.
    pub day3: LazyVec<Height, Day3, Height, Timestamp>,
    /// Zero-based Bitcoin difficulty-adjustment epoch: block height divided by
    /// 2,016 using integer division.
    pub epoch: IndexVec<Height, Epoch, ReadableBoxedVec<Height, Timestamp>>,
    /// Zero-based Bitcoin subsidy-halving epoch: block height divided by 210,000
    /// using integer division.
    pub halving: IndexVec<Height, Halving, ReadableBoxedVec<Height, Timestamp>>,
    /// Zero-based ISO week containing the block's monotonic timestamp, counted
    /// from ISO week 1 of 2009.
    pub week1: RangeMapLookupVec<Height, Week1>,
    /// Zero-based UTC calendar month containing the block's monotonic timestamp,
    /// with January 2009 equal to 0.
    pub month1: RangeMapLookupVec<Height, Month1>,
    /// Zero-based UTC calendar quarter containing the block's monotonic
    /// timestamp, with Q1 2009 equal to 0.
    pub month3: RangeMapLookupVec<Height, Month3>,
    /// Zero-based UTC calendar half-year containing the block's monotonic
    /// timestamp, with the first half of 2009 equal to 0.
    pub month6: RangeMapLookupVec<Height, Month6>,
    /// Zero-based UTC calendar year containing the block's monotonic timestamp,
    /// with 2009 equal to 0.
    pub year1: RangeMapLookupVec<Height, Year1>,
    /// Zero-based ten-year UTC period containing the block's monotonic timestamp,
    /// with 2009 through 2018 equal to 0.
    pub year10: RangeMapLookupVec<Height, Year10>,
    /// Number of transactions in the indexed block, including coinbase.
    pub tx_index_count: LazyPreviousDeltaVec<Height, Count>,
}

impl Vecs {
    /// First day affected by a height recomputation, falling back to the last
    /// indexed block when the starting height is just beyond the current tip.
    pub fn recompute_day(&self, starting_height: Height) -> Option<Day1> {
        self.day1.collect_one(starting_height).or_else(|| {
            starting_height
                .decremented()
                .and_then(|height| self.day1.collect_one(height))
        })
    }

    pub(crate) fn from_timestamps<T: VecValue>(
        name: &str,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        compute: fn(Height, Timestamp) -> T,
    ) -> LazyVec<Height, T, Height, Timestamp> {
        LazyVec::init(name, Version::ZERO, timestamps, compute)
    }

    pub fn day1_from_timestamp(timestamp: Timestamp) -> Day1 {
        Day1::try_from(Date::from(timestamp)).unwrap()
    }

    pub fn month1_from_timestamp(timestamp: Timestamp) -> Month1 {
        Month1::from(Self::day1_from_timestamp(timestamp))
    }

    pub fn week1_from_timestamp(timestamp: Timestamp) -> Week1 {
        Week1::from(Self::day1_from_timestamp(timestamp))
    }

    pub fn month3_from_timestamp(timestamp: Timestamp) -> Month3 {
        Month3::from(Self::month1_from_timestamp(timestamp))
    }

    pub fn month6_from_timestamp(timestamp: Timestamp) -> Month6 {
        Month6::from(Self::month1_from_timestamp(timestamp))
    }

    pub fn year1_from_timestamp(timestamp: Timestamp) -> Year1 {
        Year1::from(Self::month1_from_timestamp(timestamp))
    }

    pub fn year10_from_timestamp(timestamp: Timestamp) -> Year10 {
        Year10::from(Self::year1_from_timestamp(timestamp))
    }
}
