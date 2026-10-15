use std::mem;

/// The 0-based rank of percentile `p` (0.0-1.0) among `len` sorted values: the nearest rank
/// `max(0, ceil(len * p) - 1)`, the smallest value with at least `p` of the values at or below
/// it, as the cost-basis percentiles take it. `p` counts in whole parts per million, so a
/// product that lands on an integer stays exact.
#[inline]
pub fn nearest_rank(len: usize, p: f64) -> usize {
    const PPM: u64 = 1_000_000;
    let ppm = (p * PPM as f64).round() as u64;
    ((len as u64 * ppm).div_ceil(PPM) as usize).saturating_sub(1)
}

/// Exact order-statistics multiset backed by sqrt-decomposed sorted blocks.
///
/// Insert, remove, and rank lookup are O(sqrt(n)). Bulk construction sorts
/// once, then partitions the values into sorted blocks.
#[derive(Clone)]
pub struct ExactOrderStats {
    blocks: Vec<Vec<f64>>,
    len: usize,
    block_size: usize,
}

impl ExactOrderStats {
    pub fn new(expected_len: usize) -> Self {
        let block_size = (expected_len.isqrt() * 3).max(64);
        Self {
            blocks: Vec::new(),
            len: 0,
            block_size,
        }
    }

    pub fn from_unsorted(mut values: Vec<f64>) -> Self {
        values.sort_unstable_by(f64::total_cmp);
        Self::from_sorted(values)
    }

    pub fn from_sorted(values: Vec<f64>) -> Self {
        debug_assert!(
            values
                .windows(2)
                .all(|pair| !pair[0].total_cmp(&pair[1]).is_gt()),
            "order-statistics input must be sorted"
        );
        let mut stats = Self::new(values.len());
        stats.len = values.len();
        let mut values = values.into_iter();
        loop {
            let block: Vec<_> = values.by_ref().take(stats.block_size).collect();
            if block.is_empty() {
                break;
            }
            stats.blocks.push(block);
        }
        stats
    }

    /// The number of values in the set.
    pub fn count(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, value: f64) {
        self.len += 1;

        if self.blocks.is_empty() {
            self.blocks.push(vec![value]);
            return;
        }

        let block_index = self
            .blocks
            .partition_point(|block| {
                block
                    .last()
                    .is_some_and(|candidate| candidate.total_cmp(&value).is_lt())
            })
            .min(self.blocks.len() - 1);
        let block = &mut self.blocks[block_index];
        let position = block.partition_point(|candidate| candidate.total_cmp(&value).is_lt());
        block.insert(position, value);

        if block.len() > 2 * self.block_size {
            // Histories initialized empty must not retain tiny blocks as they grow.
            if self.len.isqrt() * 3 > 2 * self.block_size {
                let values = mem::take(&mut self.blocks).into_iter().flatten().collect();
                *self = Self::from_sorted(values);
            } else {
                let right = block.split_off(block.len() / 2);
                self.blocks.insert(block_index + 1, right);
            }
        }
    }

    pub fn remove(&mut self, value: f64) -> bool {
        let block_index = self.blocks.partition_point(|block| {
            block
                .last()
                .is_some_and(|candidate| candidate.total_cmp(&value).is_lt())
        });
        let Some(block) = self.blocks.get_mut(block_index) else {
            return false;
        };

        let position = block.partition_point(|candidate| candidate.total_cmp(&value).is_lt());
        if block
            .get(position)
            .is_none_or(|candidate| !candidate.total_cmp(&value).is_eq())
        {
            return false;
        }

        block.remove(position);
        self.len -= 1;
        if block.is_empty() {
            self.blocks.remove(block_index);
        }
        true
    }

    /// Percentiles (0.0-1.0, ascending) at the nearest rank ([`nearest_rank`]),
    /// resolved in one pass. The set must not be empty.
    pub fn percentiles<const N: usize>(&self, ps: [f64; N]) -> [f64; N] {
        let ranks = ps.map(|p| nearest_rank(self.len, p));
        let mut out = [0.0; N];
        self.values_at(&ranks, &mut out);
        out
    }

    /// Resolve sorted ranks in one pass through the underlying blocks.
    fn values_at(&self, ranks: &[usize], out: &mut [f64]) {
        debug_assert_eq!(ranks.len(), out.len());
        debug_assert!(ranks.windows(2).all(|pair| pair[0] <= pair[1]));
        debug_assert!(ranks.last().is_none_or(|&rank| rank < self.len));
        let mut rank_index = 0;
        let mut block_start = 0;
        for block in &self.blocks {
            let block_end = block_start + block.len();
            while rank_index < ranks.len() && ranks[rank_index] < block_end {
                out[rank_index] = block[ranks[rank_index] - block_start];
                rank_index += 1;
            }
            block_start += block.len();
            if rank_index == ranks.len() {
                return;
            }
        }
        debug_assert_eq!(rank_index, ranks.len());
    }

    pub fn count_lt(&self, value: f64) -> usize {
        let mut count = 0;
        for block in &self.blocks {
            if block
                .last()
                .is_some_and(|candidate| candidate.total_cmp(&value).is_lt())
            {
                count += block.len();
                continue;
            }
            count += block.partition_point(|candidate| candidate.total_cmp(&value).is_lt());
            break;
        }
        count
    }

    pub fn count_le(&self, value: f64) -> usize {
        let mut count = 0;
        for block in &self.blocks {
            if block
                .last()
                .is_some_and(|candidate| !candidate.total_cmp(&value).is_gt())
            {
                count += block.len();
                continue;
            }
            count += block.partition_point(|candidate| !candidate.total_cmp(&value).is_gt());
            break;
        }
        count
    }

    pub(crate) fn first(&self) -> f64 {
        self.blocks.first().unwrap().first().copied().unwrap()
    }

    pub(crate) fn last(&self) -> f64 {
        self.blocks.last().unwrap().last().copied().unwrap()
    }
}
