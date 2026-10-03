use std::{convert::Infallible, iter, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Height, Timestamp, Version};
use vecdb::{
    AnyExportableVec, AnyVec, PrintableIndex, ReadOnlyClone, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, TypedVec, short_type_name,
};

const HOUR_SECONDS: u64 = 60 * 60;
const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;
/// First backward read when locating a window start; grows 4x per step.
const WINDOW_READ_CHUNK: usize = 1 << 10;
/// Sorted requests closer than this many heights share one contiguous
/// timestamp read; farther ones are read separately.
const MAX_BRIDGED_GAP: usize = 1 << 16;

/// A storage-free window-start vector backed by one monotonic timestamp source.
///
/// Every read uses contiguous timestamp ranges and searches them in memory.
/// Point lookups into a compressed timestamp source decode a whole page per
/// probe, which made per-day binary searches over full history cost hundreds
/// of milliseconds. Range and sorted reads find the first window start once,
/// then advance it monotonically.
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

    /// Reads `[lo, index]` with contiguous range reads only, where every
    /// height below `lo` is expired relative to `index`'s timestamp.
    ///
    /// Timestamps are monotonic, so expiry is a prefix property: once the
    /// first value of a backward chunk is expired, so is everything before
    /// it. Each compressed page is decoded at most once per call.
    fn window_prefix(&self, index: usize) -> Option<(usize, Vec<Timestamp>)> {
        let mut lo = (index + 1).saturating_sub(WINDOW_READ_CHUNK);
        let mut values = self.timestamps.collect_range_dyn(lo, index + 1);
        if values.len() != index + 1 - lo {
            return None;
        }
        let current = *values.last()?;
        let mut chunk = WINDOW_READ_CHUNK;
        while lo > 0 && !self.is_expired(current, values[0]) {
            chunk = chunk.saturating_mul(4);
            let older_lo = lo.saturating_sub(chunk);
            let mut older = self.timestamps.collect_range_dyn(older_lo, lo);
            if older.len() != lo - older_lo {
                return None;
            }
            older.append(&mut values);
            values = older;
            lo = older_lo;
        }
        Some((lo, values))
    }

    /// Window start for `values[index - lo]` searched over `[0, index]`.
    /// May return `index + 1` when the window is empty (zero duration).
    fn prefix_start(&self, lo: usize, values: &[Timestamp], index: usize) -> usize {
        let current = values[index - lo];
        lo + values[..=index - lo].partition_point(|&older| self.is_expired(current, older))
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

        let Some((lo, mut values)) = self.window_prefix(from) else {
            return Ok(());
        };
        values.extend(self.timestamps.collect_range_dyn(from + 1, to));
        let to = to.min(lo + values.len());

        let mut start = self.prefix_start(lo, &values, from);
        for current in from..to {
            let timestamp = values[current - lo];
            while start < current && self.is_expired(timestamp, values[start - lo]) {
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
        if index >= self.timestamps.len() {
            return None;
        }
        let (lo, values) = self.window_prefix(index)?;
        Some(Height::from(self.prefix_start(lo, &values, index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        let len = self.timestamps.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        out.reserve(indices.len());

        // Window starts never decrease, so one start is carried across every
        // requested index, exactly as a linear scan would.
        let mut carried: Option<usize> = None;
        let mut rest = indices;
        while let Some(&first) = rest.first() {
            let mut end = 1;
            while end < rest.len() && rest[end] - rest[end - 1] <= MAX_BRIDGED_GAP {
                end += 1;
            }
            let (segment, next) = rest.split_at(end);
            rest = next;

            let Some((lo, mut values)) = self.window_prefix(first) else {
                return;
            };
            let last = segment[segment.len() - 1];
            values.extend(self.timestamps.collect_range_dyn(first + 1, last + 1));
            let available = lo + values.len();

            let fresh = self.prefix_start(lo, &values, first);
            // After a skipped gap, a linear scan would have stopped at `first`.
            let mut start = match carried {
                Some(_) => fresh.min(first),
                None => fresh,
            };
            for &current in segment {
                if current >= available {
                    return;
                }
                if start < current {
                    let timestamp = values[current - lo];
                    start += values[start - lo..current - lo]
                        .partition_point(|&older| self.is_expired(timestamp, older));
                }
                out.push(Height::from(start));
            }
            carried = Some(start);
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
