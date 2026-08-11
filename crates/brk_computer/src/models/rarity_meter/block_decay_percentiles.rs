use brk_types::StoredF32;

use crate::internal::algo::FenwickTree;

pub(super) const START_HEIGHT: usize = 210_000;
const HALF_LIFE_BLOCKS: usize = 210_000;
const BUCKET_WIDTH: f64 = 0.001;
const MAX_RATIO: f64 = 43.0;
const TREE_SIZE: usize = (MAX_RATIO / BUCKET_WIDTH) as usize + 1;

#[derive(Clone)]
pub(crate) struct BlockDecayPercentiles {
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
    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn reset(&mut self) {
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

    pub(super) fn add_bulk(&mut self, start_height: usize, values: &[StoredF32]) {
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
    pub(super) fn add(&mut self, height: usize, value: f32) {
        self.len += 1;
        if value.is_nan() {
            return;
        }
        let weight = Self::weight(height);
        self.mass += weight;
        self.tree.add(Self::to_bucket(value), &weight);
    }

    pub(super) fn quantiles<const N: usize>(&self, qs: &[f64; N], out: &mut [f64; N]) {
        if self.mass == 0.0 {
            out.fill(0.0);
            return;
        }

        let mut targets = [0.0; N];
        for (index, &quantile) in qs.iter().enumerate() {
            targets[index] = (quantile * self.mass).next_down().max(0.0);
        }

        let mut buckets = [0; N];
        self.tree
            .kth(&targets, &|weight: &f64| *weight, &mut buckets);
        for (index, bucket) in buckets.iter().enumerate() {
            out[index] = *bucket as f64 * BUCKET_WIDTH;
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::StoredF32;

    use super::{BlockDecayPercentiles, HALF_LIFE_BLOCKS, START_HEIGHT};

    fn quantile(percentiles: &BlockDecayPercentiles, quantile: f64) -> f64 {
        let mut out = [0.0; 8];
        percentiles.quantiles(&[quantile; 8], &mut out);
        out[0]
    }

    #[test]
    fn basic_quantiles() {
        let mut percentiles = BlockDecayPercentiles::default();
        for index in 1..=1000 {
            percentiles.add(START_HEIGHT + index, index as f32 / 1000.0);
        }
        assert_eq!(percentiles.len(), 1000);
        assert!((quantile(&percentiles, 0.5) - 0.5).abs() < 0.01);
        assert!((quantile(&percentiles, 0.99) - 0.99).abs() < 0.01);
        assert!((quantile(&percentiles, 0.01) - 0.01).abs() < 0.01);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(quantile(&BlockDecayPercentiles::default(), 0.5), 0.0);
    }

    #[test]
    fn one_half_life_doubles_relative_weight() {
        let mut percentiles = BlockDecayPercentiles::default();
        percentiles.add(START_HEIGHT, 1.0);
        percentiles.add(START_HEIGHT + HALF_LIFE_BLOCKS, 2.0);
        assert!((percentiles.mass - 3.0).abs() < f64::EPSILON);
        assert_eq!(quantile(&percentiles, 0.5), 2.0);
    }

    #[test]
    fn bulk_recovery_matches_incremental_state() {
        let values: Vec<_> = (0..1000)
            .map(|index| StoredF32::from(index as f64 / 100.0))
            .collect();
        let mut incremental = BlockDecayPercentiles::default();
        for (offset, value) in values.iter().enumerate() {
            incremental.add(START_HEIGHT + offset, **value);
        }

        let mut recovered = BlockDecayPercentiles::default();
        recovered.add_bulk(START_HEIGHT, &values);

        let quantiles = [0.0001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999, 0.9999];
        let mut incremental_out = [0.0; 8];
        let mut recovered_out = [0.0; 8];
        incremental.quantiles(&quantiles, &mut incremental_out);
        recovered.quantiles(&quantiles, &mut recovered_out);

        assert_eq!(incremental.len, recovered.len);
        assert!((incremental.mass - recovered.mass).abs() < 1e-9);
        assert_eq!(incremental_out, recovered_out);
    }
}
