// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::fmt::{Debug, Formatter, Result};

use crate::{Slice, ValueType, key::InternalKey};

/// Internal representation of KV pairs
#[derive(Clone, Copy)]
pub struct InternalValue<K = Slice, V = Slice> {
    /// Internal key
    pub key: InternalKey<K>,

    /// User-defined value - an arbitrary byte array
    ///
    /// Supports up to 2^32 bytes
    pub value: V,
}

impl InternalValue {
    /// Creates a new [`Value`].
    ///
    /// # Panics
    ///
    /// Panics if the key length is empty or greater than 2^16, or the value length is greater than 2^32.
    pub fn new<V: Into<Slice>>(key: InternalKey, value: V) -> Self {
        let value = value.into();

        assert!(!key.user_key.is_empty(), "key may not be empty");
        assert!(
            u32::try_from(value.len()).is_ok(),
            "values can be 2^32 bytes in length"
        );

        Self { key, value }
    }

    /// Creates a new [`Value`].
    ///
    /// # Panics
    ///
    /// Panics if the key length is empty or greater than 2^16, or the value length is greater than 2^32.
    pub fn from_components<K: Into<Slice>, V: Into<Slice>>(
        user_key: K,
        value: V,
        seqno: u64,
        value_type: ValueType,
    ) -> Self {
        let key = InternalKey::new(user_key, seqno, value_type);
        Self::new(key, value)
    }

    /// Creates a new tombstone.
    ///
    /// # Panics
    ///
    /// Panics if the key length is empty or greater than 2^16.
    pub fn new_tombstone<K: Into<Slice>>(key: K, seqno: u64) -> Self {
        let key = InternalKey::new(key, seqno, ValueType::Tombstone);
        Self::new(key, vec![])
    }

    /// Creates a new weak tombstone.
    ///
    /// # Panics
    ///
    /// Panics if the key length is empty or greater than 2^16.
    pub fn new_weak_tombstone<K: Into<Slice>>(key: K, seqno: u64) -> Self {
        let key = InternalKey::new(key, seqno, ValueType::WeakTombstone);
        Self::new(key, vec![])
    }
}

impl<K, V> InternalValue<K, V> {
    #[doc(hidden)]
    #[must_use]
    pub fn is_tombstone(&self) -> bool {
        self.key.is_tombstone()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
impl PartialEq for InternalValue {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<K: Debug, V: Debug + AsRef<[u8]>> Debug for InternalValue<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{:?} => {:?}",
            self.key,
            if self.value.as_ref().len() >= 100 {
                format!("[ ... {} bytes ]", self.value.as_ref().len())
            } else {
                format!("{:?}", self.value)
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn pik_cmp_user_key() {
        let a = InternalKey::new(*b"a", 0, ValueType::Value);
        let b = InternalKey::new(*b"b", 0, ValueType::Value);
        assert!(a < b);
    }

    #[test]
    fn pik_cmp_seqno() {
        let a = InternalKey::new(*b"a", 0, ValueType::Value);
        let b = InternalKey::new(*b"a", 1, ValueType::Value);
        assert!(a > b);
    }
}
