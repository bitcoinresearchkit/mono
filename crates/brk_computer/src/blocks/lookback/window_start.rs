use std::sync::Arc;

use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Height, Timestamp, Version};
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, PrintableIndex, ReadOnlyClone, ReadableVec, TypedVec,
    short_type_name,
};

const HOUR_SECONDS: u64 = 60 * 60;
const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;

/// A storage-free window-start vector backed by one pinned monotonic timestamp
/// cache.
///
/// Range and sorted reads find the first window start once, then advance it
/// monotonically as current heights advance.
#[derive(Clone)]
pub struct LazyWindowStartVec {
    name: Arc<str>,
    version: Version,
    duration_seconds: u64,
    timestamps: CachedBoxedVec<Height, Timestamp>,
}

impl LazyWindowStartVec {
    pub(super) fn hours(
        name: &str,
        version: Version,
        hours: u64,
        timestamps: CachedBoxedVec<Height, Timestamp>,
    ) -> Self {
        Self::new(name, version, hours * HOUR_SECONDS, timestamps)
    }

    pub(super) fn days(
        name: &str,
        version: Version,
        days: u64,
        timestamps: CachedBoxedVec<Height, Timestamp>,
    ) -> Self {
        Self::new(name, version, days * DAY_SECONDS, timestamps)
    }

    fn new(
        name: &str,
        version: Version,
        duration_seconds: u64,
        timestamps: CachedBoxedVec<Height, Timestamp>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            duration_seconds,
            timestamps,
        }
    }

    #[inline]
    fn is_expired(&self, current: Timestamp, older: Timestamp) -> bool {
        u64::from(current).saturating_sub(u64::from(older)) >= self.duration_seconds
    }

    fn start_at(&self, timestamps: &[Timestamp], index: usize) -> usize {
        let current = timestamps[index];
        timestamps[..=index].partition_point(|&older| self.is_expired(current, older))
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Height) -> Result<(), E>,
    ) -> Result<(), E> {
        let timestamps = self.timestamps.snapshot();
        let to = to.min(timestamps.len());
        if from >= to {
            return Ok(());
        }

        let mut start = self.start_at(&timestamps, from);
        for current in from..to {
            while start < current && self.is_expired(timestamps[current], timestamps[start]) {
                start += 1;
            }
            each(Height::from(start))?;
        }
        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Height)) {
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
        let timestamps = self.timestamps.snapshot();
        (index < timestamps.len()).then(|| Height::from(self.start_at(&timestamps, index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        let Some(&first) = indices.first() else {
            return;
        };
        let timestamps = self.timestamps.snapshot();
        if first >= timestamps.len() {
            return;
        }

        let mut start = self.start_at(&timestamps, first);
        out.reserve(indices.len());
        for &current in indices {
            if current >= timestamps.len() {
                break;
            }
            while start < current && self.is_expired(timestamps[current], timestamps[start]) {
                start += 1;
            }
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
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<Height, Height, _>(self)
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::RwLock;
    use vecdb::{CachedReadableVec, CachedVec, ReadableVec};

    use super::*;
    use crate::blocks::lookback::CachedWindowStartVec;

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

    fn timestamp_fixture() -> (TimestampVec, CachedVec<TimestampVec>) {
        let timestamps = TimestampVec::new([
            0,
            12 * HOUR_SECONDS as u32,
            DAY_SECONDS as u32,
            (DAY_SECONDS + 12 * HOUR_SECONDS) as u32,
            (2 * DAY_SECONDS) as u32,
        ]);
        let cached = CachedVec::wrap(timestamps.clone());
        (timestamps, cached)
    }

    fn lazy_window(
        cached_timestamps: &CachedVec<TimestampVec>,
        duration_seconds: u64,
    ) -> LazyWindowStartVec {
        LazyWindowStartVec::new(
            "lookback",
            Version::ONE,
            duration_seconds,
            cached_timestamps.cached_boxed_clone(),
        )
    }

    #[test]
    fn exact_window_boundary_is_excluded() {
        let (_, cached_timestamps) = timestamp_fixture();
        let day = lazy_window(&cached_timestamps, DAY_SECONDS);
        let hour = lazy_window(&cached_timestamps, HOUR_SECONDS);

        assert_eq!(day.collect(), [0_usize, 0, 1, 2, 3].map(Height::from));
        assert_eq!(hour.collect(), [0_usize, 1, 2, 3, 4].map(Height::from));
    }

    #[test]
    fn range_random_and_sorted_reads_match() {
        let (_, cached_timestamps) = timestamp_fixture();
        let window = CachedWindowStartVec::new(lazy_window(&cached_timestamps, DAY_SECONDS));

        assert_eq!(
            window.collect_range_at(1, 4),
            [0_usize, 1, 2].map(Height::from)
        );
        assert_eq!(window.collect_one_at(4), Some(Height::from(3_usize)));
        assert_eq!(
            window.read_sorted_at(&[0, 2, 4]),
            [0_usize, 1, 3].map(Height::from)
        );
    }

    #[test]
    fn explicit_invalidations_refresh_same_length_reorgs() {
        let (timestamps, cached_timestamps) = timestamp_fixture();
        let window = CachedWindowStartVec::new(lazy_window(&cached_timestamps, DAY_SECONDS));

        assert_eq!(window.collect_one_at(4), Some(Height::from(3_usize)));
        timestamps.replace(4, (DAY_SECONDS + 18 * HOUR_SECONDS) as u32);

        assert_eq!(window.collect_one_at(4), Some(Height::from(3_usize)));
        cached_timestamps.invalidate();
        window.invalidate();
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
        let cached = CachedVec::wrap(timestamps);

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
