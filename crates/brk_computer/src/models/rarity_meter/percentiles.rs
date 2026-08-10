use brk_traversable::Traversable;
use brk_types::{RarityPercentileId, StoredF32};

use crate::internal::algo::FenwickTree;

pub(crate) const BOUNDARY_PERCENTILE_IDS: [RarityPercentileId; 10] = [
    RarityPercentileId::Pct0_1,
    RarityPercentileId::Pct0_5,
    RarityPercentileId::Pct1,
    RarityPercentileId::Pct2,
    RarityPercentileId::Pct5,
    RarityPercentileId::Pct95,
    RarityPercentileId::Pct98,
    RarityPercentileId::Pct99,
    RarityPercentileId::Pct99_5,
    RarityPercentileId::Pct99_9,
];

pub(crate) const fn boundary_index(id: RarityPercentileId) -> Option<usize> {
    use RarityPercentileId::*;

    match id {
        Pct0_1 => Some(0),
        Pct0_5 => Some(1),
        Pct1 => Some(2),
        Pct2 => Some(3),
        Pct5 => Some(4),
        Pct95 => Some(5),
        Pct98 => Some(6),
        Pct99 => Some(7),
        Pct99_5 => Some(8),
        Pct99_9 => Some(9),
        _ => None,
    }
}

pub(crate) const fn is_lower_boundary(id: RarityPercentileId) -> bool {
    matches!(
        id,
        RarityPercentileId::Pct0_1
            | RarityPercentileId::Pct0_5
            | RarityPercentileId::Pct1
            | RarityPercentileId::Pct2
            | RarityPercentileId::Pct5
    )
}

#[derive(Clone, Traversable)]
pub struct RarityPercentiles<T> {
    pub pct0_1: T,
    pub pct0_5: T,
    pub pct1: T,
    pub pct2: T,
    pub pct5: T,
    pub pct10: T,
    pub pct20: T,
    pub pct30: T,
    pub pct40: T,
    pub pct50: T,
    pub pct60: T,
    pub pct70: T,
    pub pct80: T,
    pub pct90: T,
    pub pct95: T,
    pub pct98: T,
    pub pct99: T,
    pub pct99_5: T,
    pub pct99_9: T,
}

impl<T> RarityPercentiles<T> {
    pub(crate) fn from_fn(mut f: impl FnMut(RarityPercentileId) -> T) -> Self {
        use RarityPercentileId::*;

        Self {
            pct0_1: f(Pct0_1),
            pct0_5: f(Pct0_5),
            pct1: f(Pct1),
            pct2: f(Pct2),
            pct5: f(Pct5),
            pct10: f(Pct10),
            pct20: f(Pct20),
            pct30: f(Pct30),
            pct40: f(Pct40),
            pct50: f(Pct50),
            pct60: f(Pct60),
            pct70: f(Pct70),
            pct80: f(Pct80),
            pct90: f(Pct90),
            pct95: f(Pct95),
            pct98: f(Pct98),
            pct99: f(Pct99),
            pct99_5: f(Pct99_5),
            pct99_9: f(Pct99_9),
        }
    }

    pub(crate) fn get(&self, id: RarityPercentileId) -> &T {
        use RarityPercentileId::*;

        match id {
            Pct0_1 => &self.pct0_1,
            Pct0_5 => &self.pct0_5,
            Pct1 => &self.pct1,
            Pct2 => &self.pct2,
            Pct5 => &self.pct5,
            Pct10 => &self.pct10,
            Pct20 => &self.pct20,
            Pct30 => &self.pct30,
            Pct40 => &self.pct40,
            Pct50 => &self.pct50,
            Pct60 => &self.pct60,
            Pct70 => &self.pct70,
            Pct80 => &self.pct80,
            Pct90 => &self.pct90,
            Pct95 => &self.pct95,
            Pct98 => &self.pct98,
            Pct99 => &self.pct99,
            Pct99_5 => &self.pct99_5,
            Pct99_9 => &self.pct99_9,
        }
    }

    pub(crate) fn boundary_refs(&self) -> [&T; 10] {
        BOUNDARY_PERCENTILE_IDS.map(|id| self.get(id))
    }
}

pub(crate) const fn suffix(id: RarityPercentileId) -> &'static str {
    use RarityPercentileId::*;

    match id {
        Pct0_1 => "pct0_1",
        Pct0_5 => "pct0_5",
        Pct1 => "pct1",
        Pct2 => "pct2",
        Pct5 => "pct5",
        Pct10 => "pct10",
        Pct20 => "pct20",
        Pct30 => "pct30",
        Pct40 => "pct40",
        Pct50 => "pct50",
        Pct60 => "pct60",
        Pct70 => "pct70",
        Pct80 => "pct80",
        Pct90 => "pct90",
        Pct95 => "pct95",
        Pct98 => "pct98",
        Pct99 => "pct99",
        Pct99_5 => "pct99_5",
        Pct99_9 => "pct99_9",
    }
}

pub(crate) const fn price_suffix(id: RarityPercentileId) -> &'static str {
    use RarityPercentileId::*;

    match id {
        Pct1 => "pct01",
        Pct2 => "pct02",
        Pct5 => "pct05",
        _ => suffix(id),
    }
}

/// First block included in the Rarity Meter distribution.
pub const START_HEIGHT: usize = 210_000;

/// Number of blocks after which an observation has half the weight of a new one.
pub const HALF_LIFE_BLOCKS: usize = 210_000;

/// Block-decayed percentile tracker backed by a Fenwick tree.
///
/// An observation at `height` receives weight
/// `2 ^ ((height - START_HEIGHT) / HALF_LIFE_BLOCKS)`. Multiplying every
/// observation by the same current-height decay factor does not change
/// quantiles, so this fixed scale is exactly equivalent to halving old weights
/// every 210,000 blocks without rescaling the tree on every block.
#[derive(Clone)]
pub(crate) struct BlockDecayPercentiles {
    tree: FenwickTree<f64>,
    len: usize,
    mass: f64,
}

const BUCKET_WIDTH: f64 = 0.001;
const MAX_RATIO: f64 = 43.0;
const TREE_SIZE: usize = (MAX_RATIO / BUCKET_WIDTH) as usize + 1;

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

    /// Rebuild historical state in O(n + N).
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

    /// Add the observation for one block. O(log N).
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

    /// Compute sorted quantiles in one shared tree walk.
    pub fn quantiles<const N: usize>(&self, qs: &[f64; N], out: &mut [f64; N]) {
        if self.mass == 0.0 {
            out.fill(0.0);
            return;
        }

        let mut targets = [0.0; N];
        for (i, &q) in qs.iter().enumerate() {
            targets[i] = (q * self.mass).next_down().max(0.0);
        }

        let mut buckets = [0; N];
        self.tree
            .kth(&targets, &|weight: &f64| *weight, &mut buckets);
        for (i, bucket) in buckets.iter().enumerate() {
            out[i] = *bucket as f64 * BUCKET_WIDTH;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_preserves_existing_names_and_boundaries() {
        use RarityPercentileId::*;

        let percentiles = RarityPercentiles::from_fn(suffix);
        assert_eq!(percentiles.pct0_1, "pct0_1");
        assert_eq!(percentiles.pct50, "pct50");
        assert_eq!(percentiles.pct99_9, "pct99_9");
        assert_eq!(price_suffix(Pct1), "pct01");
        assert_eq!(price_suffix(Pct2), "pct02");
        assert_eq!(price_suffix(Pct5), "pct05");

        for (index, id) in BOUNDARY_PERCENTILE_IDS.into_iter().enumerate() {
            assert_eq!(boundary_index(id), Some(index));
        }
        assert!(boundary_index(Pct50).is_none());
    }

    fn quantile(percentiles: &BlockDecayPercentiles, q: f64) -> f64 {
        let mut out = [0.0; 8];
        percentiles.quantiles(&[q, q, q, q, q, q, q, q], &mut out);
        out[0]
    }

    #[test]
    fn basic_quantiles() {
        let mut percentiles = BlockDecayPercentiles::default();
        for i in 1..=1000 {
            percentiles.add(START_HEIGHT + i, i as f32 / 1000.0);
        }
        assert_eq!(percentiles.len(), 1000);

        let median = quantile(&percentiles, 0.5);
        assert!((median - 0.5).abs() < 0.01, "median was {median}");

        let p99 = quantile(&percentiles, 0.99);
        assert!((p99 - 0.99).abs() < 0.01, "p99 was {p99}");

        let p01 = quantile(&percentiles, 0.01);
        assert!((p01 - 0.01).abs() < 0.01, "p01 was {p01}");
    }

    #[test]
    fn empty() {
        let percentiles = BlockDecayPercentiles::default();
        assert_eq!(quantile(&percentiles, 0.5), 0.0);
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
            .map(|i| StoredF32::from(i as f64 / 100.0))
            .collect();
        let mut incremental = BlockDecayPercentiles::default();
        for (offset, value) in values.iter().enumerate() {
            incremental.add(START_HEIGHT + offset, **value);
        }

        let mut recovered = BlockDecayPercentiles::default();
        recovered.add_bulk(START_HEIGHT, &values);

        let qs = [0.0001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999, 0.9999];
        let mut incremental_out = [0.0; 8];
        let mut recovered_out = [0.0; 8];
        incremental.quantiles(&qs, &mut incremental_out);
        recovered.quantiles(&qs, &mut recovered_out);

        assert_eq!(incremental.len, recovered.len);
        assert!((incremental.mass - recovered.mass).abs() < 1e-9);
        assert_eq!(incremental_out, recovered_out);
    }
}
