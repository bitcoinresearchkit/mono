use crate::{InternalValue, Result, Slice, ValueType};
use std::iter::Peekable;

/// Retains only the latest value for each key while merging tables.
pub struct CompactionStream<I: Iterator<Item = Result<InternalValue>>> {
    inner: Peekable<I>,
    evict_tombstones: bool,
}

impl<I: Iterator<Item = Result<InternalValue>>> CompactionStream<I> {
    /// Creates a stream over sorted internal values.
    #[must_use]
    pub fn new(iter: I) -> Self {
        Self {
            inner: iter.peekable(),
            evict_tombstones: false,
        }
    }

    /// Drops tombstones after the merge reaches the last level.
    #[must_use]
    pub fn evict_tombstones(mut self, evict: bool) -> Self {
        self.evict_tombstones = evict;
        self
    }

    fn drain_key(&mut self, key: &Slice) -> Result<()> {
        loop {
            let Some(next) = self.inner.next_if(|item| match item {
                Ok(item) => item.key.user_key == *key,
                Err(_) => true,
            }) else {
                return Ok(());
            };

            next?;
        }
    }
}

impl<I: Iterator<Item = Result<InternalValue>>> Iterator for CompactionStream<I> {
    type Item = Result<InternalValue>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let head = fail_iter!(self.inner.next()?);

            if let Some(next) = self.inner.peek() {
                let Ok(next) = next else {
                    return self.inner.next();
                };

                if next.key.user_key == head.key.user_key {
                    if head.key.value_type == ValueType::Tombstone && self.evict_tombstones {
                        fail_iter!(self.drain_key(&head.key.user_key));
                        continue;
                    }

                    let drop_weak_tombstone = next.key.value_type == ValueType::Value
                        && head.key.value_type == ValueType::WeakTombstone;
                    fail_iter!(self.drain_key(&head.key.user_key));

                    if drop_weak_tombstone {
                        continue;
                    }
                } else if head.is_tombstone() && self.evict_tombstones {
                    continue;
                }
            } else if head.is_tombstone() && self.evict_tombstones {
                continue;
            }

            return Some(Ok(head));
        }
    }
}
