use crate::{InternalValue, RecordBytes, Result, Slice, ValueType};
use std::iter::Peekable;

/// Retains only the latest value for each key while merging tables.
pub struct CompactionStream<I: Iterator<Item = Result<InternalValue<K, V>>>, K = Slice, V = Slice> {
    inner: Peekable<I>,
    evict_tombstones: bool,
}

impl<K: RecordBytes, V: RecordBytes, I: Iterator<Item = Result<InternalValue<K, V>>>>
    CompactionStream<I, K, V>
{
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
}

/// What the versions of one key in a merge leave for the levels below the merge.
#[derive(Clone, Copy)]
enum Outcome {
    Nothing,
    Value,
    /// A value, with a weak tombstone pending for the next older value below the merge.
    ValueOverWeak,
    Weak,
    Tombstone,
}

impl Outcome {
    const ALL: [Self; 5] = [
        Self::Nothing,
        Self::Value,
        Self::ValueOverWeak,
        Self::Weak,
        Self::Tombstone,
    ];

    /// Reads `table` (laid out like `ALL`) at `self`.
    fn apply(self, table: [Self; 5]) -> Self {
        let [nothing, value, value_over_weak, weak, tombstone] = table;
        match self {
            Self::Nothing => nothing,
            Self::Value => value,
            Self::ValueOverWeak => value_over_weak,
            Self::Weak => weak,
            Self::Tombstone => tombstone,
        }
    }

    /// The outcome once a newer version of the key goes on top of `self`.
    fn push(self, value_type: ValueType) -> Self {
        match (value_type, self) {
            (ValueType::Tombstone, _) => Self::Tombstone,
            // A weak tombstone cancels the value under it; with none in the merge it still cancels the
            // next older value below.
            (ValueType::WeakTombstone, Self::Value) => Self::Nothing,
            (ValueType::WeakTombstone, _) => Self::Weak,
            // A value hides what is under it, but keeps a pending weak tombstone for the levels below.
            (
                ValueType::Value | ValueType::ValueOverWeakTombstone,
                Self::ValueOverWeak | Self::Weak,
            )
            | (ValueType::ValueOverWeakTombstone, Self::Nothing) => Self::ValueOverWeak,
            (ValueType::Value | ValueType::ValueOverWeakTombstone, _) => Self::Value,
        }
    }
}

impl<K: RecordBytes, V: RecordBytes, I: Iterator<Item = Result<InternalValue<K, V>>>> Iterator
    for CompactionStream<I, K, V>
{
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut head = fail_iter!(self.inner.next()?);

            // The older versions of the key, folded oldest first: compose their pushes newest first
            // (`below`, read at `o`, is the outcome of pushing them all, oldest first, onto `o`).
            let mut below = Outcome::ALL;
            while let Some(next) = self.inner.next_if(|item| match item {
                Ok(item) => item.key.user_key == head.key.user_key,
                Err(_) => true,
            }) {
                let value_type = fail_iter!(next).key.value_type;
                below = Outcome::ALL.map(|outcome| outcome.push(value_type).apply(below));
            }

            head.key.value_type = match Outcome::Nothing.apply(below).push(head.key.value_type) {
                Outcome::Nothing => continue,
                Outcome::Value => ValueType::Value,
                // Nothing lies below the last level.
                Outcome::ValueOverWeak if self.evict_tombstones => ValueType::Value,
                Outcome::ValueOverWeak => ValueType::ValueOverWeakTombstone,
                Outcome::Weak | Outcome::Tombstone if self.evict_tombstones => continue,
                Outcome::Weak => ValueType::WeakTombstone,
                Outcome::Tombstone => ValueType::Tombstone,
            };
            return Some(Ok(head));
        }
    }
}
