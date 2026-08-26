// Copyright (c) 2026-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use super::HeapItem;
use crate::{InternalValue, Result};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, binary_heap::PeekMut},
    mem,
};

/// Merges multiple KV iterators in ascending order.
pub struct ForwardMerger<I> {
    iterators: Vec<I>,
    heap: BinaryHeap<Reverse<HeapItem>>,
    initialized: bool,
}

impl<I: Iterator<Item = Result<InternalValue>>> ForwardMerger<I> {
    #[must_use]
    pub fn new(iterators: Vec<I>) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(iterators.len()),
            iterators,
            initialized: false,
        }
    }

    fn initialize(&mut self) -> Result<()> {
        for (index, iterator) in self.iterators.iter_mut().enumerate() {
            if let Some(item) = iterator.next() {
                self.heap.push(Reverse(HeapItem {
                    iterator_index: index,
                    value: item?,
                }));
            }
        }
        self.initialized = true;
        Ok(())
    }
}

impl<I: Iterator<Item = Result<InternalValue>>> Iterator for ForwardMerger<I> {
    type Item = Result<InternalValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized {
            fail_iter!(self.initialize());
        }

        let mut root = self.heap.peek_mut()?;
        let iterator_index = root.0.iterator_index;

        #[expect(clippy::indexing_slicing, reason = "we trust the HeapItem index")]
        match self.iterators[iterator_index].next() {
            Some(Ok(next_item)) => {
                let item = mem::replace(&mut root.0.value, next_item);
                Some(Ok(item))
            }
            Some(Err(error)) => {
                // Match Merger semantics: a source error replaces the current item.
                let _discarded = PeekMut::pop(root);
                Some(Err(error))
            }
            None => {
                let item = PeekMut::pop(root).0.value;
                Some(Ok(item))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValueType::Value;
    use test_log::test;

    #[test]
    fn merges_in_ascending_order() -> Result<()> {
        let first =
            ["a", "c", "e"].map(|key| Ok(InternalValue::from_components(key, b"", 0, Value)));
        let second =
            ["b", "d", "f"].map(|key| Ok(InternalValue::from_components(key, b"", 0, Value)));

        let items = ForwardMerger::new(vec![first.into_iter(), second.into_iter()])
            .collect::<Result<Vec<_>>>()?;
        let keys = items
            .iter()
            .map(|item| item.key.user_key.as_ref())
            .collect::<Vec<_>>();

        assert_eq!([b"a", b"b", b"c", b"d", b"e", b"f"], keys.as_slice());

        Ok(())
    }
}
