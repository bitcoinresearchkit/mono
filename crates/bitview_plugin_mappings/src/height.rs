use bitview_traversable::Traversable;
use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, StoredU64, Timestamp, Version, Week1, Year1, Year10,
};
use vecdb::{IndexVec, LazyVec, ReadableBoxedVec, ReadableVec, VecValue};

use bitview_vecs::{LazyPreviousDeltaVec, RangeMapLookupVec};

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
    pub tx_index_count: LazyPreviousDeltaVec<Height, StoredU64>,
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

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    };

    use brk_types::Date;
    use parking_lot::RwLock;
    use vecdb::{
        AnyVec, PrintableIndex, ReadBounds, ReadableCloneableVec, ReadableVec, TypedVec, VecIndex,
        short_type_name,
    };

    use super::*;
    use crate::resolution::ResolutionVecs;

    #[derive(Clone)]
    struct TimestampVec(Arc<RwLock<Vec<Timestamp>>>, Arc<AtomicBool>);

    impl TimestampVec {
        fn new(values: impl IntoIterator<Item = Timestamp>) -> Self {
            Self(
                Arc::new(RwLock::new(values.into_iter().collect())),
                Arc::default(),
            )
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
            assert!(
                !self.1.load(Relaxed),
                "published lookup read raw timestamps"
            );
            let values = self.0.read();
            let to = to.min(values.len());
            if from < to {
                buf.extend_from_slice(&values[from..to]);
            }
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Timestamp)) {
            assert!(
                !self.1.load(Relaxed),
                "published lookup read raw timestamps"
            );
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
            assert!(
                !self.1.load(Relaxed),
                "published lookup read raw timestamps"
            );
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
            assert!(
                !self.1.load(Relaxed),
                "published lookup read raw timestamps"
            );
            let values = self.0.read();
            values[from.min(values.len())..to.min(values.len())]
                .iter()
                .copied()
                .try_fold(init, fold)
        }
    }

    fn check_resident_period<I: VecIndex + VecValue>(convert: fn(Height, Timestamp) -> I) {
        let timestamps = [
            Timestamp::from(Date::new(2009, 1, 3)),
            Timestamp::from(Date::new(2009, 1, 3)),
            Timestamp::from(Date::new(2010, 1, 1)),
            Timestamp::from(Date::new(2019, 1, 1)),
        ];
        let expected = timestamps.map(|timestamp| convert(Height::ZERO, timestamp));
        let source = TimestampVec::new(timestamps);
        let blocked = source.1.clone();
        let raw = LazyVec::init(
            "guarded",
            Version::ONE,
            source.read_only_boxed_clone(),
            convert,
        );
        let resolution = ResolutionVecs::new(&raw);
        let reader = resolution.height_lookup().clone();
        blocked.store(true, Relaxed);
        assert_eq!(reader.collect(), expected);
        assert_eq!(reader.collect_one_at(2), Some(expected[2]));
        assert_eq!(
            reader.read_sorted_at(&[0, 0, 2, 3, 4]),
            [expected[0], expected[0], expected[2], expected[3]]
        );
        let mut bounds = ReadBounds::new();
        bounds.set("height", 3);
        bounds.scope(|| {
            assert_eq!(reader.collect_range_at(0, 100), expected[..3]);
            assert_eq!(reader.collect_one_at(3), None);
        });
    }

    #[test]
    fn every_calendar_reverse_view_uses_only_resident_boundaries() {
        check_resident_period(|_, t| Vecs::day1_from_timestamp(t));
        check_resident_period(|_, t| Vecs::week1_from_timestamp(t));
        check_resident_period(|_, t| Vecs::month1_from_timestamp(t));
        check_resident_period(|_, t| Vecs::month3_from_timestamp(t));
        check_resident_period(|_, t| Vecs::month6_from_timestamp(t));
        check_resident_period(|_, t| Vecs::year1_from_timestamp(t));
        check_resident_period(|_, t| Vecs::year10_from_timestamp(t));
    }

    #[test]
    fn fixed_height_periods_never_read_timestamps() {
        let timestamps = LazyVec::init(
            "metadata",
            Version::ONE,
            TimestampVec::new(vec![Timestamp::default(); 420_001]).read_only_boxed_clone(),
            |_: Height, _| -> Timestamp { panic!("arithmetic mapping must not read timestamps") },
        );
        let epoch = IndexVec::new("epoch", Version::ZERO, timestamps.clone(), Epoch::from);
        let halving = IndexVec::new("halving", Version::ZERO, timestamps, Halving::from);
        for index in [0usize, 2_015, 2_016, 209_999, 210_000, 420_000] {
            assert_eq!(
                epoch.collect_one_at(index),
                Some(Epoch::from(Height::from(index)))
            );
            assert_eq!(
                halving.collect_one_at(index),
                Some(Halving::from(Height::from(index)))
            );
        }
        let mut bounds = ReadBounds::new();
        bounds.set("height", 2_016);
        bounds.scope(|| {
            assert_eq!(epoch.collect_one_at(2_016), None);
            assert_eq!(halving.collect_one_at(2_016), None);
            assert_eq!(
                epoch.read_sorted_at(&[0, 2_015, 2_016]),
                [Epoch::from(0usize); 2]
            );
        });
    }
}
