/// Exact order-statistics multiset backed by sqrt-decomposed sorted blocks.
///
/// Insert, remove, and rank lookup are O(sqrt(n)). Bulk construction sorts
/// once, then partitions the values into sorted blocks.
#[derive(Clone)]
pub(crate) struct ExactOrderStats {
    blocks: Vec<Vec<f64>>,
    len: usize,
    block_size: usize,
}

impl ExactOrderStats {
    pub(crate) fn new(capacity: usize) -> Self {
        let block_size = ((capacity as f64).sqrt() as usize).max(64);
        Self {
            blocks: Vec::new(),
            len: 0,
            block_size,
        }
    }

    pub(crate) fn from_unsorted(mut values: Vec<f64>) -> Self {
        values.sort_unstable_by(f64::total_cmp);
        Self::from_sorted(values)
    }

    pub(crate) fn from_sorted(values: Vec<f64>) -> Self {
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

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn insert(&mut self, value: f64) {
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
            let right = block.split_off(block.len() / 2);
            self.blocks.insert(block_index + 1, right);
        }
    }

    pub(crate) fn remove(&mut self, value: f64) -> bool {
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

    pub(crate) fn kth(&self, mut index: usize) -> f64 {
        for block in &self.blocks {
            if index < block.len() {
                return block[index];
            }
            index -= block.len();
        }
        unreachable!("order-statistics index out of bounds")
    }

    /// Resolve sorted ranks in one pass through the underlying blocks.
    pub(crate) fn values_at(&self, ranks: &[usize], out: &mut [f64]) {
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

    pub(crate) fn count_lt(&self, value: f64) -> usize {
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

    pub(crate) fn count_le(&self, value: f64) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_build_and_rank_queries_preserve_duplicates() {
        let stats = ExactOrderStats::from_unsorted(vec![3.0, 1.0, 2.0, 2.0]);

        assert_eq!(stats.len(), 4);
        assert_eq!(stats.kth(0), 1.0);
        assert_eq!(stats.kth(1), 2.0);
        assert_eq!(stats.kth(2), 2.0);
        assert_eq!(stats.kth(3), 3.0);
        assert_eq!(stats.count_lt(2.0), 1);
        assert_eq!(stats.count_le(2.0), 3);
        let mut values = [0.0; 5];
        stats.values_at(&[0, 0, 2, 3, 3], &mut values);
        assert_eq!(values, [1.0, 1.0, 2.0, 3.0, 3.0]);
    }

    #[test]
    fn insert_and_remove_update_order_statistics() {
        let mut stats = ExactOrderStats::new(4);
        for value in [3.0, 1.0, 2.0, 2.0] {
            stats.insert(value);
        }

        assert!(stats.remove(2.0));
        assert!(!stats.remove(4.0));
        assert_eq!(stats.len(), 3);
        assert_eq!(stats.kth(0), 1.0);
        assert_eq!(stats.kth(1), 2.0);
        assert_eq!(stats.kth(2), 3.0);
    }

    #[test]
    fn total_order_distinguishes_signed_zero() {
        let stats = ExactOrderStats::from_unsorted(vec![0.0, -0.0]);

        assert_eq!(stats.kth(0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(stats.kth(1).to_bits(), 0.0_f64.to_bits());
        assert_eq!(stats.count_lt(0.0), 1);
        assert_eq!(stats.count_le(-0.0), 1);
    }
}
