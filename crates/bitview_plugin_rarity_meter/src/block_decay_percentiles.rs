use brk_types::StoredF32;

use bitview_compute::FenwickTree;

pub const START_HEIGHT: usize = 210_000;
const HALF_LIFE_BLOCKS: usize = 210_000;
const BUCKET_WIDTH: f64 = 0.001;
const MAX_RATIO: f64 = 43.0;
const TREE_SIZE: usize = (MAX_RATIO / BUCKET_WIDTH) as usize + 1;

#[derive(Clone)]
pub struct BlockDecayPercentiles {
    tree: FenwickTree<f64>,
    len: usize,
    mass: f64,
}

impl Default for BlockDecayPercentiles {
    fn default() -> Self {
        Self {
            tree: FenwickTree::new(TREE_SIZE),
            len: 0,
            mass: 0.0,
        }
    }
}

impl BlockDecayPercentiles {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reset(&mut self) {
        self.tree.reset();
        self.len = 0;
        self.mass = 0.0;
    }

    #[inline]
    fn to_bucket(value: f32) -> usize {
        (value as f64 / BUCKET_WIDTH)
            .round()
            .clamp(0.0, (TREE_SIZE - 1) as f64) as usize
    }

    #[inline]
    fn weight(height: usize) -> f64 {
        2.0_f64.powf(height.saturating_sub(START_HEIGHT) as f64 / HALF_LIFE_BLOCKS as f64)
    }

    pub fn add_bulk(&mut self, start_height: usize, values: &[StoredF32]) {
        for (offset, &value) in values.iter().enumerate() {
            self.len += 1;
            let value = *value;
            if value.is_nan() {
                continue;
            }
            let weight = Self::weight(start_height + offset);
            self.mass += weight;
            self.tree.add_raw(Self::to_bucket(value), &weight);
        }
        self.tree.build_in_place();
    }

    #[inline]
    pub fn add(&mut self, height: usize, value: f32) {
        self.len += 1;
        if value.is_nan() {
            return;
        }
        let weight = Self::weight(height);
        self.mass += weight;
        self.tree.add(Self::to_bucket(value), &weight);
    }

    pub fn quantiles<const N: usize>(&self, qs: &[f64; N], out: &mut [f64; N]) {
        if self.mass == 0.0 {
            out.fill(0.0);
            return;
        }

        let mut targets = [0.0; N];
        for (index, &quantile) in qs.iter().enumerate() {
            targets[index] = (quantile * self.mass).next_down().max(0.0);
        }

        let buckets = self.tree.kth(targets, &|weight: &f64| *weight);
        for (index, bucket) in buckets.iter().enumerate() {
            out[index] = *bucket as f64 * BUCKET_WIDTH;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::LN_2;

    use super::*;

    #[test]
    fn bulk_and_incremental_quantiles_match_a_weighted_sorted_scan() {
        // Explicit rounded buckets keep the reference independent of to_bucket.
        let observations = [
            (3.0014, 3.001),
            (1.0006, 1.001),
            (43.001, 43.0),
            (f32::NAN, f64::NAN),
            (0.0014, 0.001),
            (3.0006, 3.001),
        ];
        let start = START_HEIGHT + HALF_LIFE_BLOCKS + 17;
        let values: Vec<_> = (0..1000)
            .map(|index| StoredF32::from(observations[index % observations.len()].0))
            .collect();
        let mut weighted: Vec<_> = (0..values.len())
            .filter_map(|index| {
                let bucket = observations[index % observations.len()].1;
                let weight =
                    (LN_2 * (start + index - START_HEIGHT) as f64 / HALF_LIFE_BLOCKS as f64).exp();
                bucket.is_finite().then_some((bucket, weight))
            })
            .collect();
        weighted.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        let mass: f64 = weighted.iter().map(|&(_, weight)| weight).sum();
        let qs = [0.0001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999, 0.9999];
        let expected = qs.map(|q| {
            let mut cumulative = 0.0;
            weighted
                .iter()
                .find(|&&(_, weight)| {
                    cumulative += weight;
                    cumulative >= q * mass
                })
                .unwrap()
                .0
        });
        let mut bulk = BlockDecayPercentiles::default();
        bulk.add_bulk(start, &values);
        let mut incremental = BlockDecayPercentiles::default();
        for (offset, value) in values.iter().enumerate() {
            incremental.add(start + offset, **value);
        }
        for state in [bulk, incremental] {
            assert_eq!(state.len(), values.len());
            assert!((state.mass - mass).abs() < mass * 1e-12);
            let mut actual = [0.0; 8];
            state.quantiles(&qs, &mut actual);
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-12);
            }
        }
    }
}
