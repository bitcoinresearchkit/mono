use bitview_compute::FenwickTree;
use bitview_primitives::PriceRatio;

pub const START_HEIGHT: usize = 210_000;
const HALF_LIFE_BLOCKS: usize = 210_000;
const BUCKET_WIDTH: f64 = 0.001;
const MAX_RATIO: f64 = 43.0;
const TREE_SIZE: usize = (MAX_RATIO / BUCKET_WIDTH) as usize + 1;

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
    fn to_bucket(value: f64) -> usize {
        (value / BUCKET_WIDTH)
            .round()
            .clamp(0.0, (TREE_SIZE - 1) as f64) as usize
    }

    #[inline]
    fn weight(height: usize) -> f64 {
        2.0_f64.powf((height - START_HEIGHT) as f64 / HALF_LIFE_BLOCKS as f64)
    }

    pub fn add_bulk(&mut self, start_height: usize, values: &[PriceRatio]) {
        for (offset, &value) in values.iter().enumerate() {
            self.len += 1;
            let value = f64::from(value);
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
    pub fn add(&mut self, height: usize, value: f64) {
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
            out.fill(f64::NAN);
            return;
        }

        let mut targets = [0.0; N];
        for (index, &quantile) in qs.iter().enumerate() {
            targets[index] = (quantile * self.mass).next_down();
        }

        let buckets = self.tree.kth(targets, &|weight: &f64| *weight);
        for ((out, bucket), &quantile) in out.iter_mut().zip(buckets).zip(qs) {
            *out = (bucket as f64 + outer_edge(quantile)) * BUCKET_WIDTH;
        }
    }
}

/// Where a quantile reads in its bucket: the outer edge on either side of the median, so a
/// value crosses it only from a more extreme bucket. Within the quantile's own bucket,
/// rounding alone would decide (a lone observation read beyond its own quantiles).
fn outer_edge(quantile: f64) -> f64 {
    if quantile < 0.5 {
        -0.5
    } else if quantile > 0.5 {
        0.5
    } else {
        0.0
    }
}
