// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::{
    InternalValue, RecordBytes, Result, Slice,
    double_ended_peekable::{DoubleEndedPeekable, DoubleEndedPeekableExt},
};

/// Consumes a stream of KVs and emits a new stream according to MVCC and tombstone rules
///
/// This iterator is used for read operations.
pub struct MvccStream<
    I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>,
    K = Slice,
    V = Slice,
> {
    inner: DoubleEndedPeekable<Result<InternalValue<K, V>>, I>,
}

impl<K: RecordBytes, V: RecordBytes, I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>>
    MvccStream<I, K, V>
{
    /// Initializes a new multi-version-aware iterator.
    #[must_use]
    pub fn new(iter: I) -> Self {
        Self {
            inner: iter.double_ended_peekable(),
        }
    }

    // Drains all entries for the given user key from the front of the iterator.
    fn drain_key_min(&mut self, key: &K) -> Result<()> {
        loop {
            let Some(next) = self.inner.next_if(|kv| {
                if let Ok(kv) = kv {
                    &kv.key.user_key == key
                } else {
                    true
                }
            }) else {
                return Ok(());
            };

            next?;
        }
    }
}

impl<K: RecordBytes, V: RecordBytes, I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>>
    Iterator for MvccStream<I, K, V>
{
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        let head = fail_iter!(self.inner.next()?);

        // As long as items are the same key, ignore them
        fail_iter!(self.drain_key_min(&head.key.user_key));

        Some(Ok(head))
    }
}

impl<K: RecordBytes, V: RecordBytes, I: DoubleEndedIterator<Item = Result<InternalValue<K, V>>>>
    DoubleEndedIterator for MvccStream<I, K, V>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let tail = fail_iter!(self.inner.next_back()?);

            let prev = match self.inner.peek_back() {
                Some(Ok(prev)) => prev,
                Some(Err(_)) => {
                    #[expect(
                        clippy::expect_used,
                        reason = "we just asserted, the peeked value is an error"
                    )]
                    return Some(Err(self
                        .inner
                        .next_back()
                        .expect("should exist")
                        .expect_err("should be error")));
                }
                None => {
                    return Some(Ok(tail));
                }
            };

            if prev.key.user_key < tail.key.user_key {
                return Some(Ok(tail));
            }
        }
    }
}
