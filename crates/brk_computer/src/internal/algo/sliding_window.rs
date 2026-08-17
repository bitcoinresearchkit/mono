use super::order_statistics::ExactOrderStats;

/// Sorted sliding window for rolling distribution/median computations.
///
/// Uses sqrt-decomposition for O(sqrt(n)) insert/remove/kth instead of
/// O(n) memmoves with a flat sorted Vec.
pub(crate) struct SlidingWindowSorted {
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
        let len = self.sorted.len();
        if len == 0 {
            return 0.0;
        }
        if len == 1 {
            return self.sorted.kth(0);
        }
        let rank = p * (len - 1) as f64;
        let lo = rank.floor() as usize;
        let hi = rank.ceil() as usize;
        if lo == hi {
            self.sorted.kth(lo)
        } else {
            let frac = rank - lo as f64;
            self.sorted.kth(lo) * (1.0 - frac) + self.sorted.kth(hi) * frac
        }
    }

    /// Extract multiple percentiles in a single pass through the sorted blocks.
    /// Percentiles must be sorted ascending. Returns interpolated values.
    pub fn percentiles(&self, ps: &[f64; 5]) -> [f64; 5] {
        let len = self.sorted.len();
        if len == 0 {
            return [0.0; 5];
        }
        if len == 1 {
            return [self.sorted.kth(0); 5];
        }

        let last = (len - 1) as f64;
        let mut requests = [(0, 0); 10];
        let mut fractions = [0.0; 5];

        for (i, &p) in ps.iter().enumerate() {
            let rank = p * last;
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            requests[2 * i] = (lo, 2 * i);
            requests[2 * i + 1] = (hi, 2 * i + 1);
            fractions[i] = rank - lo as f64;
        }
        requests.sort_unstable_by_key(|request| request.0);

        let ranks = requests.map(|request| request.0);
        let mut sorted_values = [0.0; 10];
        self.sorted.values_at(&ranks, &mut sorted_values);

        let mut values = [0.0; 10];
        for ((_, destination), value) in requests.into_iter().zip(sorted_values) {
            values[destination] = value;
        }
        std::array::from_fn(|i| {
            let fraction = fractions[i];
            values[2 * i] * (1.0 - fraction) + values[2 * i + 1] * fraction
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SlidingWindowSorted;

    #[test]
    fn batched_percentiles_match_individual_queries() {
        let percentiles = [0.10, 0.25, 0.50, 0.75, 0.90];

        for values in [
            vec![],
            vec![1.0],
            vec![3.0, 1.0, 2.0, 2.0],
            (0..100).map(f64::from).collect(),
        ] {
            let mut window = SlidingWindowSorted::with_capacity(values.len());
            window.reconstruct(&values, 0, values.len());
            assert_eq!(
                window.percentiles(&percentiles),
                percentiles.map(|percentile| window.percentile(percentile))
            );
        }
    }
}
