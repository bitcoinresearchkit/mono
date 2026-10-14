use std::{
    cmp::Ordering,
    io::{Cursor, Seek, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt};
use varint_rs::{VarintReader, VarintWriter};

#[cfg(test)]
use std::mem::size_of;

#[cfg(test)]
use byteorder::LittleEndian;

#[cfg(test)]
use crate::table::block::Trailer;

use super::block::{
    Block, Decodable, Decoder, Encodable, Encoder, ParsedItem, TRAILER_START_MARKER,
};
use crate::{
    InternalValue, RecordBytes, Result, ValueType, point_read_value::PointReadValue,
    table::util::SliceIndexes,
};

// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod iter;
mod parsed_item;

pub use iter::Iter;
pub use parsed_item::DataBlockParsedItem;

impl Decodable<DataBlockParsedItem> for InternalValue {
    fn parse_restart_key<'a>(
        reader: &mut Cursor<&[u8]>,
        offset: usize,
        data: &'a [u8],
        fixed_key_len: Option<u16>,
        _fixed_value_len: Option<u32>,
    ) -> Option<(&'a [u8], u64)> {
        let value_type = unwrap!(reader.read_u8());

        if value_type == TRAILER_START_MARKER {
            return None;
        }

        let seqno = unwrap!(reader.read_u64_varint());

        let key_len: usize =
            fixed_key_len.map_or_else(|| unwrap!(reader.read_u16_varint()).into(), usize::from);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks tend to be some megabytes in size at most, so position should fit into usize"
        )]
        let key_start = offset + reader.position() as usize;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "key_len is bounded by u16::MAX, no wrap expected"
        )]
        let key_len_i64 = key_len as i64;
        unwrap!(reader.seek_relative(key_len_i64));

        let key = data.get(key_start..(key_start + key_len));

        key.map(|k| (k, seqno))
    }

    fn parse_full(
        reader: &mut Cursor<&[u8]>,
        offset: usize,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Option<DataBlockParsedItem> {
        let value_type = unwrap!(reader.read_u8());
        if value_type == TRAILER_START_MARKER {
            return None;
        }

        #[expect(clippy::expect_used, reason = "value_type is expected to be valid")]
        let value_type = ValueType::try_from(value_type).expect("should be valid value type");

        let seqno = unwrap!(reader.read_u64_varint());

        let key_len: usize =
            fixed_key_len.map_or_else(|| unwrap!(reader.read_u16_varint()).into(), usize::from);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks tend to be some megabytes in size at most, so position should fit into usize"
        )]
        let key_start = offset + reader.position() as usize;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "key_len is bounded by u16::MAX, no wrap expected"
        )]
        let key_len_i64 = key_len as i64;
        unwrap!(reader.seek_relative(key_len_i64));

        let is_value = !value_type.is_tombstone();

        let val_len: usize = if is_value {
            fixed_value_len.map_or_else(
                || unwrap!(reader.read_u32_varint()) as usize,
                |len| len as usize,
            )
        } else {
            0
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks tend to be some megabytes in size at most, so position should fit into usize"
        )]
        let val_offset = offset + reader.position() as usize;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "val_len is bounded by u32::MAX, no wrap expected"
        )]
        let val_len_i64 = val_len as i64;
        unwrap!(reader.seek_relative(val_len_i64));

        Some(if is_value {
            DataBlockParsedItem {
                value_type,
                seqno,
                prefix: None,
                key: SliceIndexes(key_start, key_start + key_len),
                value: Some(SliceIndexes(val_offset, val_offset + val_len)),
            }
        } else {
            DataBlockParsedItem {
                value_type,
                seqno,
                prefix: None,
                key: SliceIndexes(key_start, key_start + key_len),
                value: None, // TODO: enum value/tombstone, so value is not Option for values
            }
        })
    }

    fn parse_truncated(
        reader: &mut Cursor<&[u8]>,
        offset: usize,
        base_key_offset: usize,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Option<DataBlockParsedItem> {
        let value_type = unwrap!(reader.read_u8());
        if value_type == TRAILER_START_MARKER {
            return None;
        }
        let value_type = unwrap!(ValueType::try_from(value_type));

        let seqno = unwrap!(reader.read_u64_varint());

        let shared_prefix_len: usize = unwrap!(reader.read_u16_varint()).into();
        let rest_key_len: usize = fixed_key_len.map_or_else(
            || unwrap!(reader.read_u16_varint()).into(),
            |len| usize::from(len) - shared_prefix_len,
        );

        #[expect(
            clippy::cast_possible_truncation,
            reason = "truncation is not expected to happen"
        )]
        let key_offset = offset + reader.position() as usize;

        #[expect(
            clippy::cast_possible_wrap,
            reason = "rest_key_len is bounded by u16::MAX, no wrap expected"
        )]
        let rest_key_len_i64 = rest_key_len as i64;
        unwrap!(reader.seek_relative(rest_key_len_i64));

        let is_value = !value_type.is_tombstone();

        let val_len: usize = if is_value {
            fixed_value_len.map_or_else(
                || unwrap!(reader.read_u32_varint()) as usize,
                |len| len as usize,
            )
        } else {
            0
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "truncation is not expected to happen"
        )]
        let val_offset = offset + reader.position() as usize;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "val_len is bounded by u16::MAX, no wrap expected"
        )]
        let val_len_i64 = val_len as i64;
        unwrap!(reader.seek_relative(val_len_i64));

        Some(if is_value {
            DataBlockParsedItem {
                value_type,
                seqno,
                prefix: Some(SliceIndexes(
                    base_key_offset,
                    base_key_offset + shared_prefix_len,
                )),
                key: SliceIndexes(key_offset, key_offset + rest_key_len),
                value: Some(SliceIndexes(val_offset, val_offset + val_len)),
            }
        } else {
            DataBlockParsedItem {
                value_type,
                seqno,
                prefix: Some(SliceIndexes(
                    base_key_offset,
                    base_key_offset + shared_prefix_len,
                )),
                key: SliceIndexes(key_offset, key_offset + rest_key_len),
                value: None,
            }
        })
    }
}

impl<K: RecordBytes, V: RecordBytes> Encodable<()> for InternalValue<K, V> {
    fn encode_full_into<W: Write>(
        &self,
        writer: &mut W,
        _state: &mut (),
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()> {
        // We encode restart markers as:
        // [value type] [seqno] [user key len] [user key] [value len] [value]
        // 1            2       3              4          5?           6?

        writer.write_u8(u8::from(self.key.value_type))?; // 1
        writer.write_u64_varint(self.key.seqno)?; // 2

        if fixed_key_len.is_none() {
            #[expect(clippy::cast_possible_truncation, reason = "keys are u16 length max")]
            writer.write_u16_varint(self.key.user_key.as_ref().len() as u16)?; // 3
        }
        writer.write_all(self.key.user_key.as_ref())?; // 4

        // NOTE: Only write value len + value if we are actually a value
        if !self.is_tombstone() {
            if fixed_value_len.is_none() {
                #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
                writer.write_u32_varint(self.value.as_ref().len() as u32)?; // 5
            }
            writer.write_all(self.value.as_ref())?; // 6
        }

        Ok(())
    }

    fn encode_truncated_into<W: Write>(
        &self,
        writer: &mut W,
        _state: &mut (),
        shared_len: usize,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()> {
        // We encode truncated values as:
        // [value type] [seqno] [shared prefix len] [rest key len] [rest key] [value len] [value]
        // 1            2       3                   4              5          6?          7?

        writer.write_u8(u8::from(self.key.value_type))?; // 1
        writer.write_u64_varint(self.key.seqno)?; // 2

        // TODO: maybe we can skip this varint altogether if prefix truncation = false

        #[expect(clippy::cast_possible_truncation, reason = "keys are u16 length max")]
        writer.write_u16_varint(shared_len as u16)?; // 3

        let rest_len = self.key().len() - shared_len;

        if fixed_key_len.is_none() {
            #[expect(clippy::cast_possible_truncation, reason = "keys are u16 length max")]
            writer.write_u16_varint(rest_len as u16)?; // 4
        }

        #[expect(
            clippy::expect_used,
            reason = "the shared len should not be greater than key length"
        )]
        let truncated_user_key = self
            .key
            .user_key
            .as_ref()
            .get(shared_len..)
            .expect("should be in bounds");

        writer.write_all(truncated_user_key)?; // 5

        // NOTE: Only write value len + value if we are actually a value
        if !self.is_tombstone() {
            if fixed_value_len.is_none() {
                #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
                writer.write_u32_varint(self.value.as_ref().len() as u32)?; // 6
            }
            writer.write_all(self.value.as_ref())?; // 7
        }

        Ok(())
    }

    fn key(&self) -> &[u8] {
        self.key.user_key.as_ref()
    }
}

// TODO: allow disabling binary index (for meta block)
// -> saves space in metadata blocks
// -> point reads then need to use iter().find() to find stuff (which is fine)
// see https://github.com/fjall-rs/lsm-tree/issues/185

/// Block that contains key-value pairs (user data)
#[derive(Clone)]
pub struct DataBlock {
    pub inner: Block,
}

impl DataBlock {
    /// Interprets a block as a data block.
    ///
    /// The caller needs to make sure the block is actually a data block
    /// (e.g. by checking the block type, this is typically done in the `load_block` routine)
    #[must_use]
    pub fn new(inner: Block) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn point_read(&self, needle: &[u8]) -> Option<InternalValue> {
        self.point_read_item(needle)
            .map(|item| item.materialize(&self.inner.data))
    }

    pub fn point_read_value<V: RecordBytes>(
        &self,
        needle: &[u8],
    ) -> Result<Option<PointReadValue<V>>> {
        self.point_read_item(needle)
            .map(|item| item.materialize_value(&self.inner.data))
            .transpose()
    }

    fn point_read_item(&self, needle: &[u8]) -> Option<DataBlockParsedItem> {
        let mut decoder = Decoder::<InternalValue, DataBlockParsedItem>::new(&self.inner);
        if !decoder.seek(|key, _| key < needle, false) {
            return None;
        }
        for item in decoder {
            match item.compare_key(needle, &self.inner.data) {
                Ordering::Equal => return Some(item),
                Ordering::Greater => return None,
                Ordering::Less => {}
            }
        }
        None
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(
            &self.inner.data,
            Decoder::<InternalValue, DataBlockParsedItem>::new(&self.inner),
        )
    }

    /// Returns the binary index length (number of pointers).
    ///
    /// The number of pointers is equal to the number of restart intervals.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn binary_index_len(&self) -> u32 {
        let trailer = Trailer::new(&self.inner);

        // NOTE: Skip restart interval (u8) and binary index step size (u8)
        let offset = 2 * size_of::<u8>();
        let mut reader = unwrap!(trailer.as_slice().get(offset..));

        unwrap!(reader.read_u32::<LittleEndian>())
    }

    /// Returns the number of items in the block.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        Trailer::new(&self.inner).item_count()
    }

    /// Builds an data block.
    ///
    /// # Panics
    ///
    /// Panics if the given item array if empty.
    pub fn encode_into(
        writer: &mut Vec<u8>,
        items: &[InternalValue],
        restart_interval: u8,
    ) -> Result<()> {
        #[expect(clippy::expect_used, reason = "the chunk should not be empty")]
        let first_key = &items
            .first()
            .expect("chunk should not be empty")
            .key
            .user_key;

        #[expect(clippy::cast_possible_truncation, reason = "keys are u16 length max")]
        let fixed_key_len = items
            .iter()
            .map(|item| item.key.user_key.len())
            .all(|len| len == first_key.len())
            .then_some(first_key.len() as u16);

        let first_value_len = items
            .iter()
            .find(|item| !item.is_tombstone())
            .map(|item| item.value.len());
        #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
        let fixed_value_len = first_value_len.and_then(|first_len| {
            items
                .iter()
                .filter(|item| !item.is_tombstone())
                .all(|item| item.value.len() == first_len)
                .then_some(first_len as u32)
        });

        Self::encode_into_with_fixed_lengths(
            writer,
            &mut Vec::new(),
            items,
            restart_interval,
            fixed_key_len,
            fixed_value_len,
        )
    }

    pub fn encode_into_with_fixed_lengths<K: RecordBytes, V: RecordBytes>(
        writer: &mut Vec<u8>,
        restarts: &mut Vec<u32>,
        items: &[InternalValue<K, V>],
        restart_interval: u8,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()> {
        Encoder::<'_, (), InternalValue<K, V>>::new(writer, restarts, items, restart_interval)
            .use_fixed_key_len(fixed_key_len)
            .use_fixed_value_len(fixed_value_len)
            .finish()
    }
}
