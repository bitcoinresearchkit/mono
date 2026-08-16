use brk_traversable::Traversable;
use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, StoredU64, Timestamp, Version, Week1, Year1, Year10,
};
use vecdb::{CachedBoxedVec, CachedReadableVec, CachedVec, LazyVec, ReadableBoxedVec, VecValue};

use crate::internal::LazyPreviousDeltaVec;

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
    pub day1: CachedVec<LazyVec<Height, Day1, Height, Timestamp>>,
    /// Zero-based three-day UTC period containing the block's monotonic
    /// timestamp, with period 1 beginning on 2009-01-03.
    pub day3: LazyVec<Height, Day3, Height, Timestamp>,
    /// Zero-based Bitcoin difficulty-adjustment epoch: block height divided by
    /// 2,016 using integer division.
    pub epoch: LazyVec<Height, Epoch, Height, Timestamp>,
    /// Zero-based Bitcoin subsidy-halving epoch: block height divided by 210,000
    /// using integer division.
    pub halving: LazyVec<Height, Halving, Height, Timestamp>,
    /// Zero-based ISO week containing the block's monotonic timestamp, counted
    /// from ISO week 1 of 2009.
    pub week1: LazyVec<Height, Week1, Height, Timestamp>,
    /// Zero-based UTC calendar month containing the block's monotonic timestamp,
    /// with January 2009 equal to 0.
    pub month1: LazyVec<Height, Month1, Height, Timestamp>,
    /// Zero-based UTC calendar quarter containing the block's monotonic
    /// timestamp, with Q1 2009 equal to 0.
    pub month3: LazyVec<Height, Month3, Height, Timestamp>,
    /// Zero-based UTC calendar half-year containing the block's monotonic
    /// timestamp, with the first half of 2009 equal to 0.
    pub month6: LazyVec<Height, Month6, Height, Timestamp>,
    /// Zero-based UTC calendar year containing the block's monotonic timestamp,
    /// with 2009 equal to 0.
    pub year1: LazyVec<Height, Year1, Height, Timestamp>,
    /// Zero-based ten-year UTC period containing the block's monotonic timestamp,
    /// with 2009 through 2018 equal to 0.
    pub year10: LazyVec<Height, Year10, Height, Timestamp>,
    /// Number of transactions in the indexed block, including coinbase.
    pub tx_index_count: LazyPreviousDeltaVec<Height, StoredU64>,
}

impl Vecs {
    pub(crate) fn new(
        version: Version,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        transaction_count: ReadableBoxedVec<Height, StoredU64>,
    ) -> Self {
        Self {
            minute10: Self::from_timestamps("minute10", timestamps.clone(), |_, timestamp| {
                Minute10::from_timestamp(timestamp)
            }),
            minute30: Self::from_timestamps("minute30", timestamps.clone(), |_, timestamp| {
                Minute30::from_timestamp(timestamp)
            }),
            hour1: Self::from_timestamps("hour1", timestamps.clone(), |_, timestamp| {
                Hour1::from_timestamp(timestamp)
            }),
            hour4: Self::from_timestamps("hour4", timestamps.clone(), |_, timestamp| {
                Hour4::from_timestamp(timestamp)
            }),
            hour12: Self::from_timestamps("hour12", timestamps.clone(), |_, timestamp| {
                Hour12::from_timestamp(timestamp)
            }),
            day1: CachedVec::wrap(Self::from_timestamps(
                "day1",
                timestamps.clone(),
                |_, timestamp| Self::day1_from_timestamp(timestamp),
            )),
            day3: Self::from_timestamps("day3", timestamps.clone(), |_, timestamp| {
                Day3::from_timestamp(timestamp)
            }),
            epoch: Self::from_timestamps("epoch", timestamps.clone(), Self::epoch_from_height),
            halving: Self::from_timestamps(
                "halving",
                timestamps.clone(),
                Self::halving_from_height,
            ),
            week1: Self::from_timestamps("week1", timestamps.clone(), |_, timestamp| {
                Self::week1_from_timestamp(timestamp)
            }),
            month1: Self::from_timestamps("month1", timestamps.clone(), |_, timestamp| {
                Self::month1_from_timestamp(timestamp)
            }),
            month3: Self::from_timestamps("month3", timestamps.clone(), |_, timestamp| {
                Self::month3_from_timestamp(timestamp)
            }),
            month6: Self::from_timestamps("month6", timestamps.clone(), |_, timestamp| {
                Self::month6_from_timestamp(timestamp)
            }),
            year1: Self::from_timestamps("year1", timestamps.clone(), |_, timestamp| {
                Self::year1_from_timestamp(timestamp)
            }),
            year10: Self::from_timestamps("year10", timestamps, |_, timestamp| {
                Self::year10_from_timestamp(timestamp)
            }),
            tx_index_count: LazyPreviousDeltaVec::new("tx_index_count", version, transaction_count),
        }
    }

    fn from_timestamps<T: VecValue>(
        name: &str,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        compute: fn(Height, Timestamp) -> T,
    ) -> LazyVec<Height, T, Height, Timestamp> {
        LazyVec::init(name, Version::ZERO, timestamps, compute)
    }

    pub(crate) fn day1_from_timestamp(timestamp: Timestamp) -> Day1 {
        Day1::try_from(Date::from(timestamp)).unwrap()
    }

    pub(crate) fn month1_from_timestamp(timestamp: Timestamp) -> Month1 {
        Month1::from(Self::day1_from_timestamp(timestamp))
    }

    pub(crate) fn week1_from_timestamp(timestamp: Timestamp) -> Week1 {
        Week1::from(Self::day1_from_timestamp(timestamp))
    }

    pub(crate) fn month3_from_timestamp(timestamp: Timestamp) -> Month3 {
        Month3::from(Self::month1_from_timestamp(timestamp))
    }

    pub(crate) fn month6_from_timestamp(timestamp: Timestamp) -> Month6 {
        Month6::from(Self::month1_from_timestamp(timestamp))
    }

    pub(crate) fn year1_from_timestamp(timestamp: Timestamp) -> Year1 {
        Year1::from(Self::month1_from_timestamp(timestamp))
    }

    pub(crate) fn year10_from_timestamp(timestamp: Timestamp) -> Year10 {
        Year10::from(Self::year1_from_timestamp(timestamp))
    }

    fn epoch_from_height(height: Height, _: Timestamp) -> Epoch {
        Epoch::from(height)
    }

    fn halving_from_height(height: Height, _: Timestamp) -> Halving {
        Halving::from(height)
    }

    pub fn day1_read_only_boxed_clone(&self) -> ReadableBoxedVec<Height, Day1> {
        Box::new(self.day1.clone())
    }

    pub fn day1_cached_boxed_clone(&self) -> CachedBoxedVec<Height, Day1> {
        self.day1.cached_boxed_clone()
    }

    /// Invalidate mappings after monotonic timestamps are rewritten without
    /// changing length or schema version.
    pub(crate) fn invalidate_timestamp_caches(&self) {
        self.day1.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use brk_types::Date;
    use parking_lot::RwLock;
    use vecdb::{AnyVec, CachedVec, PrintableIndex, ReadableVec, TypedVec, short_type_name};

    use super::*;

    #[derive(Clone)]
    struct TimestampVec(Arc<RwLock<Vec<Timestamp>>>);

    impl TimestampVec {
        fn new(values: impl IntoIterator<Item = Timestamp>) -> Self {
            Self(Arc::new(RwLock::new(values.into_iter().collect())))
        }

        fn replace(&self, index: usize, timestamp: Timestamp) {
            self.0.write()[index] = timestamp;
        }
    }

    impl AnyVec for TimestampVec {
        fn version(&self) -> Version {
            Version::ONE
        }

        fn name(&self) -> &str {
            "timestamps"
        }

        fn len(&self) -> usize {
            self.0.read().len()
        }

        fn index_type_to_string(&self) -> &'static str {
            <Height as PrintableIndex>::to_string()
        }

        fn region_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn value_type_to_size_of(&self) -> usize {
            size_of::<Timestamp>()
        }

        fn value_type_to_string(&self) -> &'static str {
            short_type_name::<Timestamp>()
        }
    }

    impl TypedVec for TimestampVec {
        type I = Height;
        type T = Timestamp;
    }

    impl ReadableVec<Height, Timestamp> for TimestampVec {
        fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Timestamp>) {
            let values = self.0.read();
            let to = to.min(values.len());
            if from < to {
                buf.extend_from_slice(&values[from..to]);
            }
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Timestamp)) {
            let values = self.0.read();
            for &value in &values[from.min(values.len())..to.min(values.len())] {
                each(value);
            }
        }

        fn fold_range_at<B, F: FnMut(B, Timestamp) -> B>(
            &self,
            from: usize,
            to: usize,
            init: B,
            fold: F,
        ) -> B {
            let values = self.0.read();
            values[from.min(values.len())..to.min(values.len())]
                .iter()
                .copied()
                .fold(init, fold)
        }

        fn try_fold_range_at<B, E, F: FnMut(B, Timestamp) -> Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            fold: F,
        ) -> Result<B, E> {
            let values = self.0.read();
            values[from.min(values.len())..to.min(values.len())]
                .iter()
                .copied()
                .try_fold(init, fold)
        }
    }

    #[test]
    fn timestamp_mappings_match_the_previous_conversion_chain() {
        let timestamps = [
            Timestamp::from(Date::new(2009, 1, 1)),
            Timestamp::from(Date::new(2012, 4, 17)),
            Timestamp::from(Date::new(2026, 8, 3)),
        ];
        let source: ReadableBoxedVec<Height, Timestamp> = Box::new(TimestampVec::new(timestamps));

        let minute10 = Vecs::from_timestamps("minute10", source.clone(), |_, timestamp| {
            Minute10::from_timestamp(timestamp)
        });
        let day1 = Vecs::from_timestamps("day1", source.clone(), |_, timestamp| {
            Vecs::day1_from_timestamp(timestamp)
        });
        let week1 = Vecs::from_timestamps("week1", source.clone(), |_, timestamp| {
            Week1::from(Vecs::day1_from_timestamp(timestamp))
        });
        let month3 = Vecs::from_timestamps("month3", source.clone(), |_, timestamp| {
            Month3::from(Vecs::month1_from_timestamp(timestamp))
        });
        let year10 = Vecs::from_timestamps("year10", source, |_, timestamp| {
            Year10::from(Year1::from(Vecs::month1_from_timestamp(timestamp)))
        });

        assert_eq!(minute10.collect(), timestamps.map(Minute10::from_timestamp));
        assert_eq!(day1.collect(), timestamps.map(Vecs::day1_from_timestamp));
        assert_eq!(
            week1.collect(),
            timestamps.map(|timestamp| Week1::from(Vecs::day1_from_timestamp(timestamp)))
        );
        assert_eq!(
            month3.collect(),
            timestamps.map(|timestamp| Month3::from(Vecs::month1_from_timestamp(timestamp)))
        );
        assert_eq!(
            year10.collect(),
            timestamps.map(|timestamp| {
                Year10::from(Year1::from(Vecs::month1_from_timestamp(timestamp)))
            })
        );
    }

    #[test]
    fn height_mappings_preserve_epoch_boundaries() {
        let timestamp = Timestamp::from(Date::new(2009, 1, 1));

        assert_eq!(
            Vecs::epoch_from_height(Height::from(2_015_u32), timestamp),
            Epoch::from(0_usize)
        );
        assert_eq!(
            Vecs::epoch_from_height(Height::from(2_016_u32), timestamp),
            Epoch::from(1_usize)
        );
        assert_eq!(
            Vecs::halving_from_height(Height::from(209_999_u32), timestamp),
            Halving::from(0_usize)
        );
        assert_eq!(
            Vecs::halving_from_height(Height::from(210_000_u32), timestamp),
            Halving::from(1_usize)
        );
    }

    #[test]
    fn explicit_invalidation_refreshes_same_length_timestamp_changes() {
        let first = Timestamp::from(Date::new(2009, 1, 1));
        let second = Timestamp::from(Date::new(2009, 1, 2));
        let third = Timestamp::from(Date::new(2009, 1, 3));
        let timestamps = TimestampVec::new([first, second]);
        let cached_timestamps = CachedVec::wrap(timestamps.clone());
        let day1 = CachedVec::wrap(Vecs::from_timestamps(
            "day1",
            Box::new(cached_timestamps.clone()),
            |_, timestamp| Vecs::day1_from_timestamp(timestamp),
        ));

        assert_eq!(day1.collect_one_at(1), Some(Day1::from(1_usize)));
        timestamps.replace(1, third);
        assert_eq!(day1.collect_one_at(1), Some(Day1::from(1_usize)));

        cached_timestamps.invalidate();
        day1.invalidate();
        assert_eq!(day1.collect_one_at(1), Some(Day1::from(2_usize)));
    }
}
