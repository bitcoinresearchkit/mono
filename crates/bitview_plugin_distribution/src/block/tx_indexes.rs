use std::cmp::Ordering;

use brk_types::TxIndex;
use smallvec::{SmallVec, smallvec};

/// Sorted, unique transaction indexes associated with one address in a block.
#[derive(Debug)]
pub struct TxIndexes(SmallVec<[TxIndex; 4]>);

impl TxIndexes {
    #[inline]
    pub fn new(tx_index: TxIndex) -> Self {
        Self(smallvec![tx_index])
    }

    #[inline]
    pub fn push(&mut self, tx_index: TxIndex) {
        let Some(&last) = self.0.last() else {
            self.0.push(tx_index);
            return;
        };

        debug_assert!(last <= tx_index);
        if last != tx_index {
            self.0.push(tx_index);
        }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn union_len(&self, other: &Self) -> u32 {
        let mut left = self.0.iter().copied().peekable();
        let mut right = other.0.iter().copied().peekable();
        let mut count = 0;

        loop {
            match (left.peek().copied(), right.peek().copied()) {
                (Some(left_index), Some(right_index)) => {
                    count += 1;
                    match left_index.cmp(&right_index) {
                        Ordering::Less => {
                            left.next();
                        }
                        Ordering::Equal => {
                            left.next();
                            right.next();
                        }
                        Ordering::Greater => {
                            right.next();
                        }
                    }
                }
                (Some(_), None) => return count + left.count() as u32,
                (None, Some(_)) => return count + right.count() as u32,
                (None, None) => return count,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indexes(values: impl IntoIterator<Item = u32>) -> TxIndexes {
        let mut values = values.into_iter();
        let mut indexes = TxIndexes::new(TxIndex::new(values.next().unwrap()));
        for value in values {
            indexes.push(TxIndex::new(value));
        }
        indexes
    }

    #[test]
    fn push_deduplicates_consecutive_indexes() {
        let indexes = indexes([1, 1, 3, 3, 5]);

        assert_eq!(indexes.len(), 3);
    }

    #[test]
    fn union_counts_unique_indexes() {
        let left = indexes([1, 3, 3, 5]);
        let right = indexes([1, 3, 4, 4]);

        assert_eq!(left.union_len(&right), 4);
    }

    #[test]
    fn union_matches_exhaustive_small_sets() {
        const VALUES: u32 = 8;

        for left_bits in 1_u32..(1 << VALUES) {
            for right_bits in 1_u32..(1 << VALUES) {
                let left_values = (0..VALUES).filter(|value| left_bits & (1 << value) != 0);
                let right_values = (0..VALUES).filter(|value| right_bits & (1 << value) != 0);
                let expected = (left_bits | right_bits).count_ones();

                assert_eq!(
                    indexes(left_values).union_len(&indexes(right_values)),
                    expected
                );
            }
        }
    }
}
