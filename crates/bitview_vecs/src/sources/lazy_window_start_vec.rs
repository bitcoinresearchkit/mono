use std::{convert::Infallible, iter, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Height, Timestamp, Version};
use vecdb::{
    AnyExportableVec, AnyVec, PrintableIndex, ReadOnlyClone, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, TypedVec, short_type_name,
};

const HOUR_SECONDS: u64 = 60 * 60;
const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;

/// A storage-free window-start vector backed by one monotonic timestamp source.
///
/// Range and sorted reads find the first window start once, then advance it
/// monotonically. Sorted reads jump over large request gaps with binary search.
#[derive(Clone)]
pub struct LazyWindowStartVec {
    name: Arc<str>,
    version: Version,
    duration_seconds: u64,
    timestamps: ReadableBoxedVec<Height, Timestamp>,
}

impl LazyWindowStartVec {
    pub fn hours(
        name: &str,
        version: Version,
        hours: u64,
        timestamps: &(impl ReadableCloneableVec<Height, Timestamp> + ?Sized),
    ) -> Self {
        Self::new(name, version, hours * HOUR_SECONDS, timestamps)
    }

    pub fn days(
        name: &str,
        version: Version,
        days: u64,
        timestamps: &(impl ReadableCloneableVec<Height, Timestamp> + ?Sized),
    ) -> Self {
        Self::new(name, version, days * DAY_SECONDS, timestamps)
    }

    fn new(
        name: &str,
        version: Version,
        duration_seconds: u64,
        timestamps: &(impl ReadableCloneableVec<Height, Timestamp> + ?Sized),
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            duration_seconds,
            timestamps: timestamps.read_only_boxed_clone(),
        }
    }

    #[inline]
    fn is_expired(&self, current: Timestamp, older: Timestamp) -> bool {
        u64::from(current).saturating_sub(u64::from(older)) >= self.duration_seconds
    }

    fn start_at(&self, mut begin: usize, mut end: usize, current: Timestamp) -> usize {
        while begin < end {
            let middle = begin + (end - begin) / 2;
            if self.is_expired(current, self.timestamps.collect_one_at(middle).unwrap()) {
                begin = middle + 1;
            } else {
                end = middle;
            }
        }
        begin
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Height) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.timestamps.len());
        if from >= to {
            return Ok(());
        }

        let timestamps = self.timestamps.collect_range_dyn(from, to);
        let mut start = self.start_at(0, from + 1, timestamps[0]);
        let mut older = self.timestamps.cursor();
        for (offset, timestamp) in timestamps.into_iter().enumerate() {
            let current = from + offset;
            while start < current && self.is_expired(timestamp, older.get(start).unwrap()) {
                start += 1;
            }
            each(Height::from(start))?;
        }
        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Height)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }
}

impl AnyVec for LazyWindowStartVec {
    fn version(&self) -> Version {
        self.version + self.timestamps.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.timestamps.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Height>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Height>()
    }
}

impl TypedVec for LazyWindowStartVec {
    type I = Height;
    type T = Height;
}

impl ReadableVec<Height, Height> for LazyWindowStartVec {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Height>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Height)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Height) -> B>(
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

    fn try_fold_range_at<B, E, F: FnMut(B, Height) -> Result<B, E>>(
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

    fn collect_one_at(&self, index: usize) -> Option<Height> {
        let current = self.timestamps.collect_one_at(index)?;
        Some(Height::from(self.start_at(0, index + 1, current)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        let Some(&first) = indices.first() else {
            return;
        };
        let len = self.timestamps.len();
        if first >= len {
            return;
        }

        let indices = &indices[..indices.partition_point(|&index| index < len)];
        let timestamps = self.timestamps.read_sorted_at(indices);
        let mut start = self.start_at(0, first + 1, timestamps[0]);
        let mut older = self.timestamps.cursor();
        let mut previous = first;
        out.reserve(indices.len());
        for (&current, timestamp) in indices.iter().zip(timestamps) {
            // For a large request gap, binary search costs fewer comparisons
            // than walking the skipped history. Nearby requests keep the
            // existing forward scan, including its cheap duplicate handling.
            if start < current && current - previous > (current - start).ilog2() as usize + 1 {
                start = self.start_at(start, current, timestamp);
            } else {
                while start < current && self.is_expired(timestamp, older.get(start).unwrap()) {
                    start += 1;
                }
            }
            previous = current;
            out.push(Height::from(start));
        }
    }
}

impl ReadOnlyClone for LazyWindowStartVec {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl Traversable for LazyWindowStartVec {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<Height, Height, _>(self)
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::RwLock;
    use vecdb::ReadableVec;

    use super::*;

    #[derive(Clone)]
    struct TimestampVec(Arc<RwLock<Vec<Timestamp>>>);

    impl TimestampVec {
        fn new(values: impl IntoIterator<Item = u32>) -> Self {
            Self(Arc::new(RwLock::new(
                values.into_iter().map(Timestamp::from).collect(),
            )))
        }

        fn replace(&self, index: usize, value: u32) {
            self.0.write()[index] = Timestamp::from(value);
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

    fn timestamp_fixture() -> (TimestampVec, TimestampVec) {
        let timestamps = TimestampVec::new([
            0,
            12 * HOUR_SECONDS as u32,
            DAY_SECONDS as u32,
            (DAY_SECONDS + 12 * HOUR_SECONDS) as u32,
            (2 * DAY_SECONDS) as u32,
        ]);
        let cached = timestamps.clone();
        (timestamps, cached)
    }

    fn lazy_window(cached_timestamps: &TimestampVec, duration_seconds: u64) -> LazyWindowStartVec {
        LazyWindowStartVec::new(
            "lookback",
            Version::ONE,
            duration_seconds,
            cached_timestamps,
        )
    }

    #[test]
    fn sorted_gap_search_matches_linear_reference_across_duplicates_and_rewrites() {
        let mut time = 0u32;
        let mut values: Vec<_> = (0..50_000)
            .map(|i| {
                time += if i % 7 == 0 {
                    0
                } else {
                    (i % 9 * 600 + if i % 211 == 0 { 86400 } else { 0 }) as u32
                };
                time
            })
            .collect();
        let source = TimestampVec::new(values.iter().copied());
        let cached = source.clone();
        for rewrite in [false, true] {
            if rewrite {
                values[49_999] += 86_400;
                source.replace(49_999, values[49_999]);
            }
            for duration in [0, 1, HOUR_SECONDS, DAY_SECONDS, 14 * DAY_SECONDS, u64::MAX] {
                let window = lazy_window(&cached, duration);
                for indices in [
                    vec![],
                    vec![usize::MAX],
                    vec![0, 0, 10, 511, 15_000, 49_999, 50_000, usize::MAX],
                    (20_000..24_096).collect(),
                    vec![49_999, 49_999],
                ] {
                    let mut expected = Vec::new();
                    if let Some(&first) = indices.first().filter(|&&i| i < values.len()) {
                        let mut start = values[..=first].partition_point(|&old| {
                            u64::from(values[first]).saturating_sub(u64::from(old)) >= duration
                        });
                        for &current in indices.iter().take_while(|&&i| i < values.len()) {
                            while start < current
                                && u64::from(values[current])
                                    .saturating_sub(u64::from(values[start]))
                                    >= duration
                            {
                                start += 1;
                            }
                            expected.push(Height::from(start));
                        }
                    }
                    let mut actual = vec![Height::from(99usize)];
                    window.read_sorted_into_at(&indices, &mut actual);
                    assert_eq!(&actual[1..], expected, "duration={duration}");
                }
            }
        }
    }

    #[test]
    fn same_length_reorgs_are_visible_without_derived_invalidation() {
        let (timestamps, cached_timestamps) = timestamp_fixture();
        let window = lazy_window(&cached_timestamps, DAY_SECONDS);

        assert_eq!(window.collect_one_at(4), Some(Height::from(3_usize)));
        timestamps.replace(4, (DAY_SECONDS + 18 * HOUR_SECONDS) as u32);

        assert_eq!(window.collect_one_at(4), Some(Height::from(2_usize)));
    }

    #[test]
    fn range_linear_results_match_the_previous_forward_algorithm() {
        let mut timestamp = 1_230_000_000_u32;
        let values: Vec<u32> = (0..2_000)
            .map(|i| {
                let current = timestamp;
                timestamp += 60 + ((i * 997) % 7_200) as u32;
                current
            })
            .collect();
        let timestamps = TimestampVec::new(values.iter().copied());
        let cached = timestamps;

        for duration_seconds in [
            HOUR_SECONDS,
            DAY_SECONDS,
            7 * DAY_SECONDS,
            365 * DAY_SECONDS,
        ] {
            let lookback = lazy_window(&cached, duration_seconds);

            let expected: Vec<Height> = {
                let mut start = 0;
                values
                    .iter()
                    .enumerate()
                    .map(|(current, &current_timestamp)| {
                        while start < current
                            && u64::from(current_timestamp).saturating_sub(u64::from(values[start]))
                                >= duration_seconds
                        {
                            start += 1;
                        }
                        Height::from(start)
                    })
                    .collect()
            };

            assert_eq!(lookback.collect(), expected);
            assert_eq!(lookback.collect_range_at(317, 1_713), expected[317..1_713]);

            let indices = [0, 17, 318, 999, 1_712, 1_999];
            assert_eq!(
                lookback.read_sorted_at(&indices),
                indices.map(|index| expected[index])
            );
        }
    }
}
