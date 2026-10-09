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
