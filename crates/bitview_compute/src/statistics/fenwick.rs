use std::ops::SubAssign;

/// Trait for types that can be stored in a Fenwick tree.
pub trait FenwickNode: Clone + Copy + Default {
    fn add_assign(&mut self, other: &Self);
}

impl FenwickNode for u32 {
    #[inline(always)]
    fn add_assign(&mut self, other: &Self) {
        *self += other;
    }
}

impl FenwickNode for f64 {
    #[inline(always)]
    fn add_assign(&mut self, other: &Self) {
        *self += other;
    }
}

/// Generic Fenwick tree (Binary Indexed Tree) over arbitrary node types.
///
/// Uses 0-indexed buckets externally; 1-indexed internally.
/// Provides O(log N) point-update, prefix-sum, and kth walk-down.
#[derive(Clone)]
pub struct FenwickTree<N: FenwickNode> {
    /// 1-indexed tree array. Position 0 is unused.
    tree: Vec<N>,
}

impl<N: FenwickNode> FenwickTree<N> {
    pub fn new(size: usize) -> Self {
        Self {
            tree: vec![N::default(); size + 1],
        }
    }

    pub fn reset(&mut self) {
        self.tree.fill(N::default());
    }

    /// Point-update: add `delta` to the node at `bucket` (0-indexed).
    #[inline]
    pub fn add(&mut self, bucket: usize, delta: &N) {
        let mut i = bucket + 1;
        while i < self.tree.len() {
            self.tree[i].add_assign(delta);
            i += i.isolate_lowest_one();
        }
    }

    /// Prefix sum of buckets [0, bucket] inclusive (0-indexed).
    pub fn prefix_sum(&self, bucket: usize) -> N {
        let mut result = N::default();
        let mut i = bucket + 1;
        assert!(i < self.tree.len(), "Fenwick bucket out of bounds");
        while i > 0 {
            result.add_assign(&self.tree[i]);
            i -= i.isolate_lowest_one();
        }
        result
    }

    /// Find the 0-indexed bucket containing the k-th element for each target.
    ///
    /// `field_fn` extracts the relevant count field from a node.
    /// Sorted targets share node extraction while following the same path.
    ///
    /// Processes all targets at each tree level for better cache locality.
    #[inline]
    pub fn kth<V, F, const LEN: usize>(
        &self,
        sorted_targets: [V; LEN],
        field_fn: &F,
    ) -> [usize; LEN]
    where
        V: Copy + PartialOrd + SubAssign,
        F: Fn(&N) -> V,
    {
        let [out] = self.kth_many([Some(sorted_targets)], &|_, node| field_fn(node));
        out
    }

    /// Search several fields together. Absent queries return zero buckets.
    ///
    /// Searches advance at the same tree level, and adjacent targets following
    /// the same path extract their field once. No search allocates.
    #[inline]
    pub fn kth_many<V, F, const LEN: usize, const QUERIES: usize>(
        &self,
        targets: [Option<[V; LEN]>; QUERIES],
        field_fn: &F,
    ) -> [[usize; LEN]; QUERIES]
    where
        V: Copy + PartialOrd + SubAssign,
        F: Fn(usize, &N) -> V,
    {
        let len = self.tree.len();
        assert!(len > 1, "cannot search an empty Fenwick tree");
        let size = len - 1;
        let mut remaining = targets;
        let mut out = [[0; LEN]; QUERIES];
        let mut bit = 1usize << (usize::BITS - 1 - size.leading_zeros());
        while bit > 0 {
            for (query, (remaining, out)) in remaining.iter_mut().zip(&mut out).enumerate() {
                let Some(remaining) = remaining else { continue };
                let mut start = 0;
                while start < LEN {
                    let prefix = out[start];
                    let mut end = start + 1;
                    while end < LEN && out[end] == prefix {
                        end += 1;
                    }
                    let next = prefix + bit;
                    if next < len {
                        let value = field_fn(query, &self.tree[next]);
                        for (remaining, out) in
                            remaining[start..end].iter_mut().zip(&mut out[start..end])
                        {
                            if *remaining >= value {
                                *remaining -= value;
                                *out = next;
                            }
                        }
                    }
                    start = end;
                }
            }
            bit >>= 1;
        }
        out
    }

    /// Write a raw frequency delta at a bucket. Does NOT maintain the Fenwick invariant.
    /// Call [`Self::build_in_place`] after all raw writes.
    #[inline]
    pub fn add_raw(&mut self, bucket: usize, delta: &N) {
        let i = bucket + 1;
        assert!(i < self.tree.len(), "Fenwick bucket out of bounds");
        self.tree[i].add_assign(delta);
    }

    /// Convert raw frequencies (written via [`Self::add_raw`]) into a valid Fenwick tree. O(size).
    pub fn build_in_place(&mut self) {
        let len = self.tree.len();
        for i in 1..len {
            let parent = i + i.isolate_lowest_one();
            if parent < len {
                let child = self.tree[i];
                self.tree[parent].add_assign(&child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn batched_searches_share_repeated_targets_and_skip_absent_fields() {
        let mut tree = FenwickTree::<u32>::new(13);
        let frequencies = [3, 0, 2, 0, 0, 5, 1, 0, 7, 0, 0, 0, 4];
        for (bucket, frequency) in frequencies.into_iter().enumerate() {
            tree.add(bucket, &frequency);
        }
        let calls = Cell::new(0);
        let result = tree.kth_many([Some([6; 8]), Some([40; 8]), None], &|q, n| {
            calls.set(calls.get() + 1);
            match q {
                0 => *n,
                1 => 2 * n,
                _ => panic!("absent query was searched"),
            }
        });
        assert_eq!(result, [[5; 8], [12; 8], [0; 8]]);
        // Two active fields share one extraction per tree level across eight ranks.
        assert!(calls.get() <= 2 * 4);

        let targets = [21, 0, 5, 2, 9, 3, 17, 4];
        let expected = targets.map(|target| {
            let mut sum = 0;
            frequencies
                .iter()
                .position(|frequency| {
                    sum += frequency;
                    sum > target
                })
                .unwrap()
        });
        assert_eq!(tree.kth(targets, &|n| *n), expected);
    }

    #[test]
    fn build_in_place_matches_add() {
        let mut tree_add = FenwickTree::<u32>::new(8);
        tree_add.add(0, &5);
        tree_add.add(2, &3);
        tree_add.add(5, &7);
        tree_add.add(7, &1);

        let mut tree_bulk = FenwickTree::<u32>::new(8);
        tree_bulk.add_raw(0, &5);
        tree_bulk.add_raw(2, &3);
        tree_bulk.add_raw(5, &7);
        tree_bulk.add_raw(7, &1);
        tree_bulk.build_in_place();

        for i in 0..8 {
            assert_eq!(
                tree_add.prefix_sum(i),
                tree_bulk.prefix_sum(i),
                "mismatch at bucket {i}"
            );
        }
    }
}
