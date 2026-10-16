use std::{convert::Infallible, sync::Arc};

use arc_swap::ArcSwap;
use brk_types::{Height, Timestamp, Version};
use vecdb::{
    AnyVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec, VecIndex,
};

const DAY_SECONDS: u64 = 24 * 60 * 60;
/// First backward read when locating a window start; grows 4x per step.
const WINDOW_READ_CHUNK: usize = 1 << 10;
/// Sorted requests closer than this many heights share one contiguous
/// timestamp read; farther ones are read separately.
const MAX_BRIDGED_GAP: usize = 1 << 16;
/// Latest heights whose window starts stay resident: an ordinary update and the
/// height before it read only these.
const TIP_HEIGHTS: usize = 8;
/// Fewest timestamps read at a time while carrying a window start forward.
const TIP_READ: usize = 64;

/// Window starts of the latest heights, computed from timestamps of `version`.
#[derive(Default)]
struct Tip {
    version: Version,
    first: usize,
    starts: Vec<Height>,
}

impl Tip {
    fn get(&self, from: usize, to: usize) -> Option<&[Height]> {
        self.starts
            .get(from.checked_sub(self.first)?..to.checked_sub(self.first)?)
    }
}

/// A storage-free window-start vector backed by one monotonic timestamp source.
///
/// Every read uses contiguous timestamp ranges and searches them in memory.
/// Point lookups into a compressed timestamp source decode a whole page per
/// probe, which made per-day binary searches over full history cost hundreds
/// of milliseconds. Range and sorted reads find the first window start once,
/// then advance it monotonically. The latest heights' starts are carried forward
/// by [`Self::update`] and shared by every clone, so an ordinary update reads
/// none of the history.
#[derive(Clone)]
pub struct LazyWindowStartVec {
    name: Arc<str>,
    version: Version,
    duration_seconds: u64,
    timestamps: ReadableBoxedVec<Height, Timestamp>,
    tip: Arc<ArcSwap<Tip>>,
}

impl LazyWindowStartVec {
    pub fn days(
        name: &str,
        version: Version,
        days: u64,
        timestamps: &(impl ReadableCloneableVec<Height, Timestamp> + ?Sized),
    ) -> Self {
        // A window holds at least its own height: its start is never past it.
        debug_assert!(days > 0, "{name}: a window spans at least one day");
        Self {
            name: Arc::from(name),
            version,
            duration_seconds: days * DAY_SECONDS,
            timestamps: timestamps.read_only_boxed_clone(),
            tip: Arc::default(),
        }
    }

    /// Republishes the latest heights' starts after the timestamps changed from
    /// `starting_height`; called by the timestamps' writer after each update.
    /// Starts below the change carry forward; anything else is recomputed.
    pub fn update(&self, starting_height: Height) {
        let len = self.timestamps.len();
        let version = self.timestamps.version();
        let first = len.saturating_sub(TIP_HEIGHTS);
        let tip = self.tip.load();
        let unchanged = if tip.version == version {
            (tip.first + tip.starts.len()).min(starting_height.to_usize())
        } else {
            0
        };
        let seed = first
            .min(unchanged)
            .checked_sub(1)
            .and_then(|height| Some((height, tip.get(height, height + 1)?[0].to_usize())));
        let mut starts = Vec::with_capacity(len - first);
        let complete = match seed {
            Some((height, start)) => self.advance(start, height + 1, len, first, &mut starts),
            None if first < len => self.window_prefix(first).and_then(|(lo, values)| {
                let start = self.prefix_start(lo, &values, first);
                starts.push(Height::from(start));
                self.advance(start, first + 1, len, first, &mut starts)
            }),
            None => Some(()),
        };
        if complete.is_none() {
            starts.clear();
        }
        self.tip.store(Arc::new(Tip {
            version,
            first,
            starts,
        }));
    }

    /// Carries `start`, the window start of `from - 1`, through `[from, to)` and
    /// pushes the starts from `keep` on, reading only the new timestamps and the
    /// ones the start moves past.
    fn advance(
        &self,
        mut start: usize,
        from: usize,
        to: usize,
        keep: usize,
        out: &mut Vec<Height>,
    ) -> Option<()> {
        let current = self.timestamps.collect_range_dyn(from, to);
        if current.len() != to - from {
            return None;
        }
        let mut passed = Vec::new();
        let mut lo = start;
        for (height, timestamp) in (from..).zip(current) {
            while start < height {
                if start == lo + passed.len() {
                    let end = (start + (to - height).max(TIP_READ)).min(height);
                    lo = start;
                    passed.clear();
                    self.timestamps.read_into_at(lo, end, &mut passed);
                    if passed.len() != end - lo {
                        return None;
                    }
                }
                if !self.is_expired(timestamp, passed[start - lo]) {
                    break;
                }
                start += 1;
            }
            if height >= keep {
                out.push(Height::from(start));
            }
        }
        Some(())
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
        if let Some(starts) = self.tip.load().get(from, to) {
            return starts.iter().try_for_each(|&start| each(start));
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
        let Ok(()) = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, Infallible>(())
        });
    }

    /// Window starts of sorted, present `indices`, searched in the timestamps; `None` once a read
    /// comes up short.
    fn search_sorted_into(&self, indices: &[usize], out: &mut Vec<Height>) -> Option<()> {
        // Window starts never decrease, so one start is carried across a segment's indices,
        // exactly as a linear scan would.
        let mut rest = indices;
        while let Some(&first) = rest.first() {
            let mut end = 1;
            while end < rest.len() && rest[end] - rest[end - 1] <= MAX_BRIDGED_GAP {
                end += 1;
            }
            let (segment, next) = rest.split_at(end);
            rest = next;

            let (lo, mut values) = self.window_prefix(first)?;
            let last = segment[segment.len() - 1];
            values.extend(self.timestamps.collect_range_dyn(first + 1, last + 1));
            let available = lo + values.len();

            let mut start = self.prefix_start(lo, &values, first);
            for &current in segment {
                if current >= available {
                    return None;
                }
                if start < current {
                    let timestamp = values[current - lo];
                    start += values[start - lo..current - lo]
                        .partition_point(|&older| self.is_expired(timestamp, older));
                }
                out.push(Height::from(start));
            }
        }
        Some(())
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

    fn is_mutable(&self) -> bool {
        self.timestamps.is_mutable()
    }

    vecdb::any_vec_type_methods!(Height, Height);
}

impl TypedVec for LazyWindowStartVec {
    type I = Height;
    type T = Height;
}

impl ReadableVec<Height, Height> for LazyWindowStartVec {
    vecdb::visitor_read_methods!(Height, for_each_value, try_for_each_value);

    fn collect_one_at(&self, index: usize) -> Option<Height> {
        if index >= self.timestamps.len() {
            return None;
        }
        if let Some(&[start]) = self.tip.load().get(index, index + 1) {
            return Some(start);
        }
        let (lo, values) = self.window_prefix(index)?;
        Some(Height::from(self.prefix_start(lo, &values, index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        let len = self.timestamps.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        out.reserve(indices.len());

        // Sorted, the requests the resident tip covers form one run: only the others search.
        let tip = self.tip.load();
        let covered_from = indices.partition_point(|&index| index < tip.first);
        let covered_to = indices.partition_point(|&index| index < tip.first + tip.starts.len());
        let (covered, after) = indices[covered_from..].split_at(covered_to - covered_from);
        if self
            .search_sorted_into(&indices[..covered_from], out)
            .is_none()
        {
            return;
        }
        out.extend(covered.iter().map(|&index| tip.starts[index - tip.first]));
        self.search_sorted_into(after, out);
    }
}

impl ReadOnlyClone for LazyWindowStartVec {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

bitview_traversable::leaf!([] LazyWindowStartVec);
