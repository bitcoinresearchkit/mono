use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Date, Height, Timestamp, Version};
use vecdb::{
    AnyExportableVec, AnyVec, CachedVec, PrintableIndex, ReadOnlyClone, ReadableBoxedVec,
    ReadableVec, TypedVec, VecIndex, short_type_name,
};

/// Pinned date lookup derived from one monotonic height-to-timestamp source.
#[derive(Clone)]
pub struct CachedDateVec<I: VecIndex>(CachedVec<DateSource<I>>);

impl<I: VecIndex> CachedDateVec<I> {
    pub(crate) fn new(
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        period_from_timestamp: fn(Timestamp) -> I,
        date_from_period_and_timestamp: fn(I, Timestamp) -> Date,
    ) -> Self {
        Self(CachedVec::wrap(DateSource::new(
            timestamps,
            period_from_timestamp,
            date_from_period_and_timestamp,
        )))
    }

    pub(crate) fn invalidate(&self) {
        self.0.invalidate();
    }

    pub fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, Date> {
        Box::new(self.clone())
    }
}

impl<I: VecIndex> AnyVec for CachedDateVec<I> {
    fn version(&self) -> Version {
        self.0.version()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.0.index_type_to_string()
    }

    fn region_names(&self) -> Vec<String> {
        self.0.region_names()
    }

    fn value_type_to_size_of(&self) -> usize {
        self.0.value_type_to_size_of()
    }

    fn value_type_to_string(&self) -> &'static str {
        self.0.value_type_to_string()
    }
}

impl<I: VecIndex> TypedVec for CachedDateVec<I> {
    type I = I;
    type T = Date;
}

impl<I: VecIndex> ReadableVec<I, Date> for CachedDateVec<I> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Date>) {
        self.0.read_into_at(from, to, buf);
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Date)) {
        self.0.for_each_range_dyn_at(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Date) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        self.0.fold_range_at(from, to, init, fold)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Date) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.0.try_fold_range_at(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<Date> {
        self.0.collect_one_at(index)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Date>) {
        self.0.read_sorted_into_at(indices, out);
    }
}

impl<I: VecIndex> ReadOnlyClone for CachedDateVec<I> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<I: VecIndex> Traversable for CachedDateVec<I> {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, Date, _>(self)
    }
}

#[derive(Clone)]
struct DateSource<I: VecIndex> {
    timestamps: ReadableBoxedVec<Height, Timestamp>,
    period_from_timestamp: fn(Timestamp) -> I,
    date_from_period_and_timestamp: fn(I, Timestamp) -> Date,
}

impl<I: VecIndex> DateSource<I> {
    fn new(
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        period_from_timestamp: fn(Timestamp) -> I,
        date_from_period_and_timestamp: fn(I, Timestamp) -> Date,
    ) -> Self {
        Self {
            timestamps,
            period_from_timestamp,
            date_from_period_and_timestamp,
        }
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Date) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(());
        }

        let chunk_size = self.timestamps.cursor_chunk_size();
        let source_len = self.timestamps.len();
        let mut source_from = 0;
        let mut target = from;
        let mut buf = Vec::with_capacity(chunk_size);
        let mut previous_period = None;

        while source_from < source_len && target < to {
            let source_to = (source_from + chunk_size).min(source_len);
            buf.clear();
            self.timestamps
                .read_into_at(source_from, source_to, &mut buf);

            for timestamp in buf.iter().copied() {
                let period = (self.period_from_timestamp)(timestamp).to_usize();
                debug_assert!(previous_period.is_none_or(|previous| previous <= period));
                previous_period = Some(period);
                while target <= period && target < to {
                    each((self.date_from_period_and_timestamp)(
                        I::from(target),
                        timestamp,
                    ))?;
                    target += 1;
                }
            }

            source_from = source_to;
        }

        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Date)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, std::convert::Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }
}

impl<I: VecIndex> AnyVec for DateSource<I> {
    fn version(&self) -> Version {
        self.timestamps.version()
    }

    fn name(&self) -> &str {
        "date"
    }

    fn len(&self) -> usize {
        self.timestamps.collect_last().map_or(0, |timestamp| {
            (self.period_from_timestamp)(timestamp).to_usize() + 1
        })
    }

    fn index_type_to_string(&self) -> &'static str {
        <I as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Date>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Date>()
    }
}

impl<I: VecIndex> TypedVec for DateSource<I> {
    type I = I;
    type T = Date;
}

impl<I: VecIndex> ReadableVec<I, Date> for DateSource<I> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Date>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Date)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Date) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Date) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::RwLock;

    use brk_types::{Day1, Day3};

    use super::*;

    #[derive(Clone)]
    struct TimestampVec(Arc<RwLock<Vec<Timestamp>>>);

    impl TimestampVec {
        fn new(values: impl IntoIterator<Item = Timestamp>) -> Self {
            Self(Arc::new(RwLock::new(values.into_iter().collect())))
        }

        fn replace(&self, index: usize, value: Timestamp) {
            self.0.write()[index] = value;
        }

        fn push(&self, value: Timestamp) {
            self.0.write().push(value);
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

    fn day1(timestamp: Timestamp) -> Day1 {
        Day1::try_from(Date::from(timestamp)).unwrap()
    }

    fn day3(timestamp: Timestamp) -> Day3 {
        Day3::from_timestamp(timestamp)
    }

    fn period_date(day: Day1, _: Timestamp) -> Date {
        Date::from(day)
    }

    fn timestamp_date(_: Day3, timestamp: Timestamp) -> Date {
        Date::from(timestamp)
    }

    #[test]
    fn day1_uses_the_period_date_across_empty_days() {
        let date = CachedDateVec::new(
            Box::new(TimestampVec::new([
                Timestamp::from(Date::new(2009, 1, 3)),
                Timestamp::from(Date::new(2009, 1, 9)),
            ])),
            day1,
            period_date,
        );

        assert_eq!(
            date.collect(),
            [
                Date::new(2009, 1, 1),
                Date::new(2009, 1, 2),
                Date::new(2009, 1, 3),
                Date::new(2009, 1, 4),
                Date::new(2009, 1, 5),
                Date::new(2009, 1, 6),
                Date::new(2009, 1, 7),
                Date::new(2009, 1, 8),
                Date::new(2009, 1, 9),
            ]
        );
    }

    #[test]
    fn coarser_period_gaps_use_the_next_periods_first_date() {
        let date = CachedDateVec::new(
            Box::new(TimestampVec::new([
                Timestamp::from(Date::new(2009, 1, 3)),
                Timestamp::from(Date::new(2009, 1, 9)),
            ])),
            day3,
            timestamp_date,
        );

        assert_eq!(
            date.collect(),
            [
                Date::new(2009, 1, 3),
                Date::new(2009, 1, 3),
                Date::new(2009, 1, 9),
                Date::new(2009, 1, 9),
            ]
        );
    }

    #[test]
    fn growing_to_a_new_period_refreshes_automatically() {
        let timestamps = TimestampVec::new([
            Timestamp::from(Date::new(2009, 1, 3)),
            Timestamp::from(Date::new(2009, 1, 9)),
        ]);
        let date = CachedDateVec::new(Box::new(timestamps.clone()), day3, timestamp_date);

        assert_eq!(date.collect_last(), Some(Date::new(2009, 1, 9)));

        timestamps.push(Timestamp::from(Date::new(2009, 1, 12)));

        assert_eq!(date.collect_last(), Some(Date::new(2009, 1, 12)));
    }

    #[test]
    fn explicit_invalidation_refreshes_same_length_rewrites() {
        let timestamps = TimestampVec::new([
            Timestamp::from(Date::new(2009, 1, 3)),
            Timestamp::from(Date::new(2009, 1, 9)),
        ]);
        let date = CachedDateVec::new(Box::new(timestamps.clone()), day3, timestamp_date);

        assert_eq!(date.collect_last(), Some(Date::new(2009, 1, 9)));

        timestamps.replace(1, Timestamp::from(Date::new(2009, 1, 10)));
        assert_eq!(date.collect_last(), Some(Date::new(2009, 1, 9)));

        date.invalidate();
        assert_eq!(date.collect_last(), Some(Date::new(2009, 1, 10)));
    }
}
