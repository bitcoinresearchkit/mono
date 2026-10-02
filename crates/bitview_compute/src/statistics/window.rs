use super::order::ExactOrderStats;

/// Sorted sliding window for rolling distribution/median computations.
///
/// Uses sqrt-decomposition for O(sqrt(n)) insert/remove/kth instead of
/// O(n) memmoves with a flat sorted Vec.
pub struct SlidingWindowSorted {
    sorted: ExactOrderStats,
    prev_start: usize,
}

impl SlidingWindowSorted {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            sorted: ExactOrderStats::new(cap),
            prev_start: 0,
        }
    }

    /// Reconstruct state from historical data (the elements in [range_start..skip]).
    /// Uses O(n log n) sort + O(n) block construction instead of O(n√n) individual inserts.
    pub fn reconstruct(&mut self, partial_values: &[f64], range_start: usize, skip: usize) {
        self.prev_start = range_start;
        let slice = &partial_values[..skip - range_start];
        if slice.is_empty() {
            return;
        }
        self.sorted = ExactOrderStats::from_unsorted(slice.to_vec());
    }

    /// Add a new value and remove all expired values up to `new_start`.
    pub fn advance(
        &mut self,
        value: f64,
        new_start: usize,
        partial_values: &[f64],
        range_start: usize,
    ) {
        self.sorted.insert(value);

        while self.prev_start < new_start {
            let old = partial_values[self.prev_start - range_start];
            self.sorted.remove(old);
            self.prev_start += 1;
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }

    #[inline]
    pub fn min(&self) -> f64 {
        if self.sorted.is_empty() {
            0.0
        } else {
            self.sorted.first()
        }
    }

    #[inline]
    pub fn max(&self) -> f64 {
        if self.sorted.is_empty() {
            0.0
        } else {
            self.sorted.last()
        }
    }

    /// Extract a percentile (0.0-1.0) using linear interpolation.
    #[inline]
    pub fn percentile(&self, p: f64) -> f64 {
        self.sorted.percentile(p)
    }

    /// Extract multiple percentiles in a single pass through the sorted blocks.
    pub fn percentiles(&self, ps: &[f64; 5]) -> [f64; 5] {
        self.sorted.percentiles(ps)
    }
}
