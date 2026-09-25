// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod forward;

pub use forward::ForwardMerger;

use crate::{InternalValue, RecordBytes, Result, Slice};
use interval_heap::IntervalHeap as Heap;
use std::cmp::Ordering;

struct HeapItem<K = Slice, V = Slice> {
    iterator_index: usize,
    value: InternalValue<K, V>,
}

impl<K: Ord, V> Eq for HeapItem<K, V> {}

impl<K: Ord, V> PartialEq for HeapItem<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.value.key == other.value.key
    }
}

impl<K: Ord, V> Ord for HeapItem<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.key.cmp(&other.value.key)
    }
}

impl<K: Ord, V> PartialOrd for HeapItem<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Merges multiple KV iterators
pub struct Merger<I, K: Ord = Slice, V = Slice> {
    iterators: Vec<I>,
    heap: Heap<HeapItem<K, V>>,
    initialized_lo: bool,
    initialized_hi: bool,
}

impl<K: RecordBytes, V: RecordBytes, I: Iterator<Item = Result<InternalValue<K, V>>>>
    Merger<I, K, V>
{
    #[must_use]
    pub fn new(iterators: Vec<I>) -> Self {
        let heap = Heap::with_capacity(if iterators.len() > 1 {
            iterators.len()
        } else {
            0
        });

        Self {
            iterators,
            heap,
            initialized_lo: false,
            initialized_hi: false,
        }
    }

    fn initialize_lo(&mut self) -> Result<()> {
        for (idx, it) in self.iterators.iter_mut().enumerate() {
            if let Some(item) = it.next() {
                let item = item?;
                self.heap.push(HeapItem {
                    iterator_index: idx,
                    value: item,
                });
            }
        }
        self.initialized_lo = true;
        Ok(())
    }
}

impl<K: RecordBytes, V: RecordBytes, I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>>
    Merger<I, K, V>
{
    fn initialize_hi(&mut self) -> Result<()> {
        for (idx, it) in self.iterators.iter_mut().enumerate() {
            if let Some(item) = it.next_back() {
                let item = item?;
                self.heap.push(HeapItem {
                    iterator_index: idx,
                    value: item,
                });
            }
        }
        self.initialized_hi = true;
        Ok(())
    }
}

impl<K: RecordBytes, V: RecordBytes, I: Iterator<Item = Result<InternalValue<K, V>>>> Iterator
    for Merger<I, K, V>
{
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized_lo {
            // A single source needs no heap; keep it on this direct path.
            if let [iterator] = self.iterators.as_mut_slice() {
                return iterator.next();
            }
            fail_iter!(self.initialize_lo());
        }

        let min_item = self.heap.pop_min()?;

        #[expect(clippy::indexing_slicing, reason = "we trust the HeapItem index")]
        if let Some(next_item) = self.iterators[min_item.iterator_index].next() {
            let next_item = fail_iter!(next_item);
            self.heap.push(HeapItem {
                iterator_index: min_item.iterator_index,
                value: next_item,
            });
        }

        Some(Ok(min_item.value))
    }
}

impl<K: RecordBytes, V: RecordBytes, I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>>
    DoubleEndedIterator for Merger<I, K, V>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.initialized_hi {
            // A single source needs no heap; keep it on this direct path.
            if let [iterator] = self.iterators.as_mut_slice() {
                return iterator.next_back();
            }
            fail_iter!(self.initialize_hi());
        }

        let max_item = self.heap.pop_max()?;

        #[expect(clippy::indexing_slicing, reason = "we trust the HeapItem index")]
        if let Some(next_item) = self.iterators[max_item.iterator_index].next_back() {
            let next_item = fail_iter!(next_item);
            self.heap.push(HeapItem {
                iterator_index: max_item.iterator_index,
                value: next_item,
            });
        }

        Some(Ok(max_item.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValueType::Value;
    use test_log::test;

    #[test]
    #[expect(clippy::unwrap_used)]
    fn merge_simple() -> Result<()> {
        #[rustfmt::skip]
        let a = vec![
            Ok(InternalValue::from_components("a", b"", 0, Value)),
        ];
        #[rustfmt::skip]
        let b = vec![
            Ok(InternalValue::from_components("b", b"", 0, Value)),
        ];

        let mut iter = Merger::new(vec![a.into_iter(), b.into_iter()]);

        assert_eq!(
            iter.next().unwrap()?,
            InternalValue::from_components("a", b"", 0, Value),
        );
        assert_eq!(
            iter.next().unwrap()?,
            InternalValue::from_components("b", b"", 0, Value),
        );
        assert!(iter.next().is_none(), "iter should be closed");

        Ok(())
    }
}
