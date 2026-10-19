use crate::{ReadableVec, VecIndex, VecValue};

/// The index of the value before a window starting at `start`: none for a window from 0, whose
/// value before is the default (zero). Window views combine a value with the one before its window,
/// so a change or a sum over a cumulative source covers `[start, h]`.
#[inline(always)]
pub fn before_window(start: usize) -> Option<usize> {
    start.checked_sub(1)
}

/// Lowest and highest index of the values before windows starting at `starts`; none when every
/// window starts at 0. Window starts need not be monotonic.
pub fn before_window_bounds<I: VecIndex>(starts: &[I]) -> Option<(usize, usize)> {
    starts
        .iter()
        .filter_map(|start| before_window(start.to_usize()))
        .fold(None, |bounds, index| {
            Some(bounds.map_or((index, index), |(low, high)| {
                (low.min(index), high.max(index))
            }))
        })
}

/// Buffers, reused across batches, for one dense source's current values and the values before
/// the windows they end.
pub struct WindowBuffers<T> {
    values: Vec<T>,
    before: Vec<T>,
}

impl<T> Default for WindowBuffers<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            before: Vec::new(),
        }
    }
}

/// One batch read through [`WindowBuffers`].
pub struct WindowBatch<'a, T> {
    pub current: &'a [T],
    before: &'a [T],
    before_from: usize,
}

impl<T: VecValue> WindowBuffers<T> {
    /// Reads `[at, end)` and the values in `bounds` (lowest and highest index, from
    /// [`before_window_bounds`]). Overlapping ranges are read once; the gap between disjoint ones
    /// never is. Every read completes before this returns: sources may hold non-reentrant
    /// publication locks while lending chunks.
    pub fn read<I: VecIndex>(
        &mut self,
        source: &(impl ReadableVec<I, T> + ?Sized),
        at: usize,
        end: usize,
        bounds: Option<(usize, usize)>,
    ) -> WindowBatch<'_, T> {
        self.values.clear();
        match bounds {
            Some((low, high)) if high + 1 >= at => {
                let from = low.min(at);
                source.read_into_at(from, end, &mut self.values);
                debug_assert_eq!(self.values.len(), end - from, "window sources are dense");
                WindowBatch {
                    current: &self.values[at - from..],
                    before: &self.values,
                    before_from: from,
                }
            }
            Some((low, high)) => {
                self.before.clear();
                source.read_into_at(low, high + 1, &mut self.before);
                source.read_into_at(at, end, &mut self.values);
                debug_assert!(
                    self.before.len() == high + 1 - low && self.values.len() == end - at,
                    "window sources are dense"
                );
                WindowBatch {
                    current: &self.values,
                    before: &self.before,
                    before_from: low,
                }
            }
            None => {
                source.read_into_at(at, end, &mut self.values);
                debug_assert_eq!(self.values.len(), end - at, "window sources are dense");
                WindowBatch {
                    current: &self.values,
                    before: &[],
                    before_from: 0,
                }
            }
        }
    }
}

impl<T: Clone + Default> WindowBatch<'_, T> {
    /// The value before the window starting at `start`: the default for a window from 0.
    #[inline(always)]
    pub fn before(&self, start: usize) -> T {
        before_window(start).map_or_else(T::default, |index| {
            self.before[index - self.before_from].clone()
        })
    }
}
