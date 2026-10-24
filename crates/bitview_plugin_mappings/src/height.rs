use std::fmt::Debug;

use bitview_primitives::{
    Count, Date, Day1, Day3, DifficultyEpoch, HalvingEpoch, Hour1, Hour4, Hour12, Minute10,
    Minute30, Month1, Month3, Month6, Week1, Year1, Year10,
};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPreviousDeltaVec;
use brk_types::{Height, Timestamp, Version};
use vecdb::{IndexVec, LazyVec, ReadableBoxedVec, VecValue};

use crate::RangeMapLookupVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based 10-minute UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub minute10: LazyVec<Height, Minute10, Timestamp>,
    /// Zero-based 30-minute UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub minute30: LazyVec<Height, Minute30, Timestamp>,
    /// Zero-based one-hour UTC period containing the block's monotonic timestamp,
    /// counted from 2009-01-01 00:00:00 UTC.
    pub hour1: LazyVec<Height, Hour1, Timestamp>,
    /// Zero-based four-hour UTC period containing the block's monotonic
    /// timestamp, counted from 2009-01-01 00:00:00 UTC.
    pub hour4: LazyVec<Height, Hour4, Timestamp>,
    /// Zero-based 12-hour UTC period containing the block's monotonic timestamp,
    /// counted from 2009-01-01 00:00:00 UTC.
    pub hour12: LazyVec<Height, Hour12, Timestamp>,
    /// Zero-based UTC calendar day containing the block's monotonic timestamp,
    /// with 2009-01-01 equal to 0.
    pub day1: RangeMapLookupVec<Height, Day1>,
    /// Zero-based three-day UTC period containing the block's monotonic
    /// timestamp, with period 1 beginning on 2009-01-03.
    pub day3: LazyVec<Height, Day3, Timestamp>,
    /// Zero-based difficulty epoch: block height divided by 2,016, rounded down.
    pub difficulty_epoch: IndexVec<Height, DifficultyEpoch, ReadableBoxedVec<Height, Timestamp>>,
    /// Zero-based halving epoch (block-subsidy era): block height divided by 210,000, rounded
    /// down; epoch zero begins at genesis.
    pub halving_epoch: IndexVec<Height, HalvingEpoch, ReadableBoxedVec<Height, Timestamp>>,
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
    /// Zero-based UTC decade containing the block's monotonic timestamp: 0 is
    /// 2009 alone, 1 is 2010 through 2019, 2 is 2020 through 2029, and so on.
    pub year10: RangeMapLookupVec<Height, Year10>,
    /// Number of transactions in the indexed block, including coinbase. Hidden:
    /// the public series is `transactions.count` (`tx_count`).
    #[traversable(hidden)]
    pub tx_index_count: LazyPreviousDeltaVec<Height, Count>,
}

impl Vecs {
    pub(crate) fn from_timestamps<T: VecValue>(
        name: &str,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        compute: fn(Height, Timestamp) -> T,
    ) -> LazyVec<Height, T, Timestamp> {
        LazyVec::new(name, Version::ZERO, timestamps, compute)
    }

    /// The calendar period holding each block's timestamp.
    pub(crate) fn dated_from_timestamps<T>(
        name: &str,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
    ) -> LazyVec<Height, T, Timestamp>
    where
        T: VecValue + TryFrom<Date, Error: Debug>,
    {
        Self::from_timestamps(name, timestamps, |_, timestamp| {
            T::try_from(Date::from(timestamp)).unwrap()
        })
    }
}
