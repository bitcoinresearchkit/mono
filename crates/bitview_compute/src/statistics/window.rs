use std::collections::VecDeque;

use vecdb::{ReadableVec, VecIndex, VecValue};

use super::order::ExactOrderStats;

/// A rolling window's values in row order and sorted: the state of the rolling distribution
/// and median kernels. Its statistics depend on the window's values alone, so a continued
/// window and a rebuilt one agree exactly.
pub struct SortedWindow {
    /// The window's values, the first at row `start`.
    values: VecDeque<f64>,
    start: usize,
    sorted: ExactOrderStats,
}

impl SortedWindow {
    /// Rebuilds the window of the row before `next` from history, none of it before row `first`
    /// (empty when that row is before it).
    pub(crate) fn read<I, A>(
        next: usize,
        first: usize,
        window_starts: &impl ReadableVec<I, I>,
        source: &impl ReadableVec<I, A>,
    ) -> Self
    where
        I: VecIndex,
        A: VecValue,
        f64: From<A>,
    {
        if next <= first {
            // (Nothing before `first`: empty, the next row to enter `first`.)
            return Self {
                values: VecDeque::new(),
                start: first,
                sorted: ExactOrderStats::from_sorted(Vec::new()),
            };
        }
        let start = window_starts
            .collect_one_at(next - 1)
            .unwrap()
            .to_usize()
            .max(first);
        let mut values = VecDeque::with_capacity(next - start);
        source.for_each_range_at(start, next, |value| values.push_back(f64::from(value)));
        let sorted = ExactOrderStats::from_unsorted(values.iter().copied().collect());
        Self {
            values,
            start,
            sorted,
        }
    }

    /// Enters row `index`'s value and drops the rows before its window `start`.
    #[inline]
    pub(crate) fn push(&mut self, index: usize, value: f64, start: usize) {
        debug_assert_eq!(index, self.start + self.values.len(), "rows enter in order");
        debug_assert!(start <= index, "a window holds its own row");
        self.values.push_back(value);
        self.sorted.insert(value);
        while self.start < start {
            let expired = self.values.pop_front().unwrap();
            let removed = self.sorted.remove(expired);
            debug_assert!(removed);
            self.start += 1;
        }
    }

    #[inline]
    pub(crate) fn min(&self) -> f64 {
        self.sorted.first()
    }

    #[inline]
    pub(crate) fn max(&self) -> f64 {
        self.sorted.last()
    }

    /// Percentiles (0.0-1.0, ascending) at the nearest rank.
    #[inline]
    pub(crate) fn percentiles<const N: usize>(&self, ps: [f64; N]) -> [f64; N] {
        self.sorted.percentiles(ps)
    }
}
