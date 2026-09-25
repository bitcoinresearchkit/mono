// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result},
};

use crate::{Slice, ValueType};

#[derive(Clone, Copy, Eq)]
pub struct InternalKey<K = Slice> {
    pub user_key: K,
    pub seqno: u64,
    pub value_type: ValueType,
}

impl<K: PartialEq> PartialEq for InternalKey<K> {
    fn eq(&self, other: &Self) -> bool {
        self.user_key == other.user_key && self.seqno == other.seqno
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<K: Debug> Debug for InternalKey<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{:?}:{}:{}",
            self.user_key,
            self.seqno,
            match self.value_type {
                ValueType::Value => "V",
                ValueType::Tombstone => "T",
                ValueType::WeakTombstone => "W",
            },
        )
    }
}

impl InternalKey {
    pub fn new<K: Into<Slice>>(user_key: K, seqno: u64, value_type: ValueType) -> Self {
        let user_key = user_key.into();

        assert!(
            u16::try_from(user_key.len()).is_ok(),
            "keys can be 65535 bytes in length",
        );

        Self {
            user_key,
            seqno,
            value_type,
        }
    }
}

impl<K> InternalKey<K> {
    pub fn is_tombstone(&self) -> bool {
        self.value_type.is_tombstone()
    }
}

impl<K: Ord> PartialOrd for InternalKey<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Order by user key ascending, THEN by sequence number descending
// This is one of the most important functions
// Otherwise queries will not match expected behaviour
impl<K: Ord> Ord for InternalKey<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.user_key, other.seqno).cmp(&(&other.user_key, self.seqno))
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn key_order_smoke_test() {
        use ValueType::Value as V;

        let mut keys = [
            InternalKey::new(b"d", 3, V),
            InternalKey::new(b"a", 0, V),
            InternalKey::new(b"b", 0, V),
            InternalKey::new(b"a", 2, V),
            InternalKey::new(b"b", 1, V),
            InternalKey::new(b"a", 1, V),
            InternalKey::new(b"c", 3, V),
        ];
        keys.sort();

        assert_eq!(
            [
                InternalKey::new(b"a", 2, V),
                InternalKey::new(b"a", 1, V),
                InternalKey::new(b"a", 0, V),
                InternalKey::new(b"b", 1, V),
                InternalKey::new(b"b", 0, V),
                InternalKey::new(b"c", 3, V),
                InternalKey::new(b"d", 3, V),
            ],
            keys,
        );
    }
}
