// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::cmp::Ordering;

use crate::{
    InternalValue, RecordBytes, Result, Slice, ValueType,
    key::InternalKey,
    point_read_value::PointReadValue,
    table::{
        block::ParsedItem,
        util::{SliceIndexes, compare_prefixed_slice},
    },
};

#[derive(Debug)]
pub struct DataBlockParsedItem {
    pub value_type: ValueType,
    pub seqno: u64,
    pub prefix: Option<SliceIndexes>,
    pub key: SliceIndexes,
    pub value: Option<SliceIndexes>,
}

impl ParsedItem<InternalValue> for DataBlockParsedItem {
    fn compare_key(&self, needle: &[u8], bytes: &[u8]) -> Ordering {
        if let Some(prefix) = &self.prefix {
            let prefix = unsafe { bytes.get_unchecked(prefix.0..prefix.1) };
            let rest_key = unsafe { bytes.get_unchecked(self.key.0..self.key.1) };
            compare_prefixed_slice(prefix, rest_key, needle)
        } else {
            let key = unsafe { bytes.get_unchecked(self.key.0..self.key.1) };
            key.cmp(needle)
        }
    }

    fn key_offset(&self) -> usize {
        self.key.0
    }

    fn materialize(&self, bytes: &Slice) -> InternalValue {
        unwrap!(self.materialize_as(bytes))
    }
}

impl DataBlockParsedItem {
    pub fn materialize_as<K: RecordBytes, V: RecordBytes>(
        &self,
        bytes: &Slice,
    ) -> Result<InternalValue<K, V>> {
        // NOTE: We consider the prefix and key slice indexes to be trustworthy
        #[expect(clippy::indexing_slicing)]
        let key = if let Some(prefix) = &self.prefix {
            let prefix_key = &bytes[prefix.0..prefix.1];
            let rest_key = &bytes[self.key.0..self.key.1];
            K::from_parts(prefix_key, rest_key)?
        } else {
            K::from_block(bytes, self.key.0..self.key.1)?
        };

        let key = InternalKey {
            user_key: key,
            seqno: self.seqno,
            value_type: self.value_type,
        };

        let value = self.materialize_value(bytes)?.value;

        Ok(InternalValue { key, value })
    }

    pub(super) fn materialize_value<V: RecordBytes>(
        &self,
        bytes: &Slice,
    ) -> Result<PointReadValue<V>> {
        let value = self.value.as_ref().map_or_else(
            || Ok(V::empty()),
            |value| V::from_block(bytes, value.0..value.1),
        )?;

        Ok(PointReadValue {
            value_type: self.value_type,
            value,
        })
    }
}
