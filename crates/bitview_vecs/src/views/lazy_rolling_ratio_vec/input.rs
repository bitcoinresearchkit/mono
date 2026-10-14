use brk_types::Height;
use vecdb::{ReadableBoxedVec, VecValue};

/// One input's current batch and the preceding values its window starts reach, in buffers
/// reused across batches. Overlapping ranges share one read; disjoint ranges skip the gap.
pub(super) struct RollingInput<T> {
    values: Vec<T>,
    previous: Vec<T>,
    /// Absolute index of the first preceding value.
    base: usize,
    /// Position of the batch's first current value in `values`.
    offset: usize,
    shared: bool,
}

impl<T> Default for RollingInput<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            previous: Vec::new(),
            base: 0,
            offset: 0,
            shared: true,
        }
    }
}

impl<T: VecValue> RollingInput<T> {
    /// Reads `[at, end)` and the preceding values in `bounds` (lowest and highest index).
    pub(super) fn read(
        &mut self,
        source: &ReadableBoxedVec<Height, T>,
        at: usize,
        end: usize,
        bounds: Option<(usize, usize)>,
    ) {
        self.values.clear();
        match bounds {
            Some((low, high)) if high + 1 >= at => {
                let from = low.min(at);
                source.read_into_at(from, end, &mut self.values);
                (self.base, self.offset, self.shared) = (from, at - from, true);
            }
            Some((low, high)) => {
                self.previous.clear();
                source.read_into_at(low, high + 1, &mut self.previous);
                source.read_into_at(at, end, &mut self.values);
                (self.base, self.offset, self.shared) = (low, 0, false);
            }
            None => {
                source.read_into_at(at, end, &mut self.values);
                (self.base, self.offset, self.shared) = (at, 0, true);
            }
        }
    }

    pub(super) fn current(&self) -> &[T] {
        &self.values[self.offset..]
    }

    pub(super) fn previous(&self, index: usize) -> T {
        let values = if self.shared {
            &self.values
        } else {
            &self.previous
        };
        values[index - self.base].clone()
    }
}
