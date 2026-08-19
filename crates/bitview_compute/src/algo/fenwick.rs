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
            i += i & i.wrapping_neg();
        }
    }

    /// Prefix sum of buckets [0, bucket] inclusive (0-indexed).
    pub fn prefix_sum(&self, bucket: usize) -> N {
        let mut result = N::default();
        let mut i = bucket + 1;
        assert!(i < self.tree.len(), "Fenwick bucket out of bounds");
        while i > 0 {
            result.add_assign(&self.tree[i]);
            i -= i & i.wrapping_neg();
        }
        result
    }

    /// Find the 0-indexed bucket containing the k-th element for each target.
    ///
    /// `field_fn` extracts the relevant count field from a node.
    /// `sorted_targets` must be sorted ascending. `out` receives the 0-indexed
    /// bucket for each target. Both slices must have the same length.
    ///
    /// Processes all targets at each tree level for better cache locality.
    #[inline]
    pub fn kth<V, F>(&self, sorted_targets: &[V], field_fn: &F, out: &mut [usize])
    where
        V: Copy + PartialOrd + std::ops::SubAssign,
        F: Fn(&N) -> V,
    {
        assert_eq!(out.len(), sorted_targets.len());
        let len = self.tree.len();
        assert!(len > 1, "cannot search an empty Fenwick tree");
        let size = len - 1;
        out.fill(0);
        // Copy targets so we can subtract in-place
        let mut remaining: smallvec::SmallVec<[V; 24]> = sorted_targets.into();
        let mut bit = 1usize << (usize::BITS - 1 - size.leading_zeros());
        while bit > 0 {
            for (remaining, out) in remaining.iter_mut().zip(out.iter_mut()) {
                let next = *out + bit;
                if next < len {
                    let val = field_fn(&self.tree[next]);
                    if *remaining >= val {
                        *remaining -= val;
                        *out = next;
                    }
                }
            }
            bit >>= 1;
        }
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
            let parent = i + (i & i.wrapping_neg());
            if parent < len {
                let child = self.tree[i];
                self.tree[parent].add_assign(&child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_and_prefix_sum() {
        let mut tree = FenwickTree::<u32>::new(10);
        tree.add(0, &3);
        tree.add(1, &2);
        tree.add(5, &7);

        assert_eq!(tree.prefix_sum(0), 3);
        assert_eq!(tree.prefix_sum(1), 5);
        assert_eq!(tree.prefix_sum(4), 5);
        assert_eq!(tree.prefix_sum(5), 12);
        assert_eq!(tree.prefix_sum(9), 12);
    }

    #[test]
    fn kth_walk_down() {
        let mut tree = FenwickTree::<u32>::new(5);
        // freq: [3, 2, 0, 5, 1]
        tree.add(0, &3);
        tree.add(1, &2);
        tree.add(3, &5);
        tree.add(4, &1);

        let mut out = [0usize; 6];
        tree.kth(&[0u32, 2, 3, 4, 5, 10], &|n: &u32| *n, &mut out);
        assert_eq!(out[0], 0); // kth(0) → bucket 0
        assert_eq!(out[1], 0); // kth(2) → bucket 0 (last of bucket 0)
        assert_eq!(out[2], 1); // kth(3) → bucket 1
        assert_eq!(out[3], 1); // kth(4) → bucket 1
        assert_eq!(out[4], 3); // kth(5) → bucket 3 (bucket 2 is empty)
        assert_eq!(out[5], 4); // kth(10) → bucket 4
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

    #[test]
    fn reset_clears_all() {
        let mut tree = FenwickTree::<u32>::new(10);
        tree.add(3, &42);
        tree.reset();
        assert_eq!(tree.prefix_sum(9), 0);
    }
}
