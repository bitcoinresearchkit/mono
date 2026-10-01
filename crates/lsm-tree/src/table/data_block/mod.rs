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
use crate::{Slice, table::block::Trailer};

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

#[cfg(test)]
mod iter_test;

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

    /// Accesses the inner raw bytes
    #[cfg(test)]
    #[must_use]
    fn as_slice(&self) -> &Slice {
        &self.inner.data
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
    fn binary_index_len(&self) -> u32 {
        let trailer = Trailer::new(&self.inner);

        // NOTE: Skip restart interval (u8) and binary index step size (u8)
        let offset = 2 * size_of::<u8>();
        let mut reader = unwrap!(trailer.as_slice().get(offset..));

        unwrap!(reader.read_u32::<LittleEndian>())
    }

    /// Returns the number of items in the block.
    #[cfg(test)]
    #[must_use]
    fn len(&self) -> usize {
        Trailer::new(&self.inner).item_count()
    }

    #[cfg(test)]
    fn encode_into_vec(items: &[InternalValue], restart_interval: u8) -> Result<Vec<u8>> {
        let mut buf = vec![];

        Self::encode_into(&mut buf, items, restart_interval)?;

        Ok(buf)
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
            items,
            restart_interval,
            fixed_key_len,
            fixed_value_len,
        )
    }

    pub fn encode_into_with_fixed_lengths<K: RecordBytes, V: RecordBytes>(
        writer: &mut Vec<u8>,
        items: &[InternalValue<K, V>],
        restart_interval: u8,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()> {
        Encoder::<'_, (), InternalValue<K, V>>::new(writer, items, restart_interval)
            .use_fixed_key_len(fixed_key_len)
            .use_fixed_value_len(fixed_value_len)
            .finish()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use test_log::test;

    use crate::{
        InternalValue, Result, Slice,
        ValueType::{self, Tombstone, Value},
        table::{
            Block, DataBlock,
            block::{BlockType, Header, ParsedItem},
        },
    };

    #[test]
    fn restart_groups_preserve_records_at_block_boundaries() -> Result<()> {
        let parts = |item: InternalValue| {
            (
                item.key.user_key,
                item.key.seqno,
                item.key.value_type,
                item.value,
            )
        };
        for interval in [1_u8, 2, 3, 4, 8, 16, 31, 255] {
            let width = u16::from(interval);
            for count in [
                1,
                width.saturating_sub(1).max(1),
                width,
                width + 1,
                2 * width + 1,
            ] {
                let items: Vec<_> = (0..count)
                    .map(|i| {
                        let (value, kind) = if i % 3 == 0 {
                            (Slice::empty(), Tombstone)
                        } else {
                            (Slice::from(i.to_be_bytes()), Value)
                        };
                        InternalValue::from_components(i.to_be_bytes(), value, u64::from(i), kind)
                    })
                    .collect();
                let bytes = DataBlock::encode_into_vec(&items, interval)?;
                let block = DataBlock::new(Block {
                    data: bytes.into(),
                    header: Header {
                        block_type: BlockType::Data,
                        data_length: 0,
                        uncompressed_length: 0,
                    },
                });
                assert_eq!(block.len(), items.len());
                assert_eq!(
                    block.binary_index_len(),
                    u32::from(count).div_ceil(u32::from(interval))
                );
                let expected: Vec<_> = items.iter().cloned().map(parts).collect();
                let forward: Vec<_> = block
                    .iter()
                    .map(|item| parts(item.materialize(block.as_slice())))
                    .collect();
                let reverse: Vec<_> = block
                    .iter()
                    .rev()
                    .map(|item| parts(item.materialize(block.as_slice())))
                    .collect();
                assert_eq!(forward, expected);
                assert_eq!(reverse, expected.into_iter().rev().collect::<Vec<_>>());
                for item in items {
                    assert_eq!(
                        block.point_read(&item.key.user_key).map(parts),
                        Some(parts(item))
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn fixed_width_block_roundtrip_is_smaller() -> Result<()> {
        let items: Vec<_> = (0..256u64)
            .map(|i| {
                InternalValue::from_components(i.to_be_bytes(), (i as u32).to_be_bytes(), 0, Value)
            })
            .collect();

        let fixed = DataBlock::encode_into_vec(&items, 16)?;
        let mut variable = Vec::new();
        DataBlock::encode_into_with_fixed_lengths(&mut variable, &items, 16, None, None)?;

        assert!(
            fixed.len() + items.len() < variable.len(),
            "fixed-width encoding should omit at least one length per item",
        );

        for bytes in [fixed, variable] {
            let block = DataBlock::new(Block {
                data: bytes.into(),
                header: Header {
                    block_type: BlockType::Data,
                    data_length: 0,
                    uncompressed_length: 0,
                },
            });
            let decoded: Vec<_> = block
                .iter()
                .map(|item| item.materialize(block.as_slice()))
                .collect();
            assert_eq!(items, decoded);
        }

        Ok(())
    }

    #[test]
    fn data_block_ping_pong_fuzz_1() -> Result<()> {
        let items = [
            InternalValue::from_components(
                Slice::from([111]),
                Slice::from([119]),
                8_602_264_972_526_186_597,
                Value,
            ),
            InternalValue::from_components(
                Slice::from([121, 120, 99]),
                Slice::from([101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101]),
                11_426_548_769_907,
                Value,
            ),
        ];

        let ping_pong_code = [1, 0];

        let bytes: Vec<u8> = DataBlock::encode_into_vec(&items, 1)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        let expected_ping_ponged_items = {
            let mut iter = items.iter();
            let mut v = vec![];

            for &x in &ping_pong_code {
                if x == 0 {
                    v.push(iter.next().cloned().expect("should have item"));
                } else {
                    v.push(iter.next_back().cloned().expect("should have item"));
                }
            }

            v
        };

        let real_ping_ponged_items = {
            let mut iter = data_block
                .iter()
                .map(|x| x.materialize(data_block.as_slice()));

            let mut v = vec![];

            for &x in &ping_pong_code {
                if x == 0 {
                    v.push(iter.next().expect("should have item"));
                } else {
                    v.push(iter.next_back().expect("should have item"));
                }
            }

            v
        };

        assert_eq!(expected_ping_ponged_items, real_ping_ponged_items);

        Ok(())
    }

    #[test]
    fn data_block_point_read_simple() -> Result<()> {
        let items = [
            InternalValue::from_components("b", "b", 0, Value),
            InternalValue::from_components("c", "c", 0, Value),
            InternalValue::from_components("d", "d", 1, Tombstone),
            InternalValue::from_components("e", "e", 0, Value),
            InternalValue::from_components("f", "f", 0, Value),
        ];

        for restart_interval in 1..=16 {
            let bytes: Vec<u8> = DataBlock::encode_into_vec(&items, restart_interval)?;

            let data_block = DataBlock::new(Block {
                data: bytes.into(),
                header: Header {
                    block_type: BlockType::Data,
                    data_length: 0,
                    uncompressed_length: 0,
                },
            });

            assert!(
                data_block.point_read(b"a").is_none(),
                "should return None because a does not exist",
            );

            assert!(
                data_block.point_read(b"b").is_some(),
                "should return Some because b exists",
            );

            assert!(
                data_block.point_read(b"z").is_none(),
                "should return Some because z does not exist",
            );
        }

        Ok(())
    }

    #[test]
    fn data_block_point_read_one() -> Result<()> {
        let items = [InternalValue::from_components(
            "pla:earth:fact",
            "eaaaaaaaaarth",
            0,
            ValueType::Value,
        )];

        let bytes = DataBlock::encode_into_vec(&items, 16)?;
        let serialized_len = bytes.len();

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());
        assert_eq!(data_block.inner.size(), serialized_len);
        assert_eq!(1, data_block.binary_index_len());

        for needle in items {
            assert_eq!(
                Some(needle.clone()),
                data_block.point_read(&needle.key.user_key),
            );
        }

        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }

    #[test]
    fn data_block_point_read_first() -> Result<()> {
        let items = [InternalValue::from_components(
            "hello",
            "world",
            0,
            ValueType::Value,
        )];

        for restart_interval in 1..=16 {
            let bytes = DataBlock::encode_into_vec(&items, restart_interval)?;
            let serialized_len = bytes.len();

            let data_block = DataBlock::new(Block {
                data: bytes.into(),
                header: Header {
                    block_type: BlockType::Data,
                    data_length: 0,
                    uncompressed_length: 0,
                },
            });

            assert_eq!(data_block.len(), items.len());
            assert_eq!(data_block.inner.size(), serialized_len);

            assert_eq!(Some(items[0].clone()), data_block.point_read(b"hello"));
        }

        Ok(())
    }

    #[test]
    fn data_block_point_read_fuzz_1() -> Result<()> {
        let items = [
            InternalValue::from_components([0], b"", 23_523_531_241_241_242, Value),
            InternalValue::from_components([1], b"", 0, Value),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 16)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());

        for needle in items {
            assert_eq!(
                Some(needle.clone()),
                data_block.point_read(&needle.key.user_key),
            );
        }

        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }

    #[test]
    fn data_block_point_read_fuzz_2() -> Result<()> {
        let items = [
            InternalValue::from_components([0], [], 5, Value),
            InternalValue::from_components([1], [], 4, Tombstone),
            InternalValue::from_components([2], [], 3, Value),
            InternalValue::from_components([3], [], 0, Value),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 2)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());

        for needle in items {
            assert_eq!(
                Some(needle.clone()),
                data_block.point_read(&needle.key.user_key),
            );
        }

        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }

    #[test]
    fn data_block_point_read_dense() -> Result<()> {
        let items = [
            InternalValue::from_components(b"a", b"a", 3, Value),
            InternalValue::from_components(b"b", b"b", 2, Value),
            InternalValue::from_components(b"c", b"c", 1, Value),
            InternalValue::from_components(b"d", b"d", 65, Value),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 1)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());
        assert_eq!(4, data_block.binary_index_len());

        for needle in items {
            assert_eq!(
                Some(needle.clone()),
                data_block.point_read(&needle.key.user_key),
            );
        }

        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }

    #[test]
    #[expect(clippy::unwrap_used)]
    fn data_block_point_read_fuzz_3() -> Result<()> {
        let items = [
            InternalValue::from_components(Slice::from([0]), Slice::from([]), 0, Value),
            InternalValue::from_components(Slice::from([233, 233]), Slice::from([]), 0, Value),
            InternalValue::from_components(
                Slice::from([255, 255, 0]),
                Slice::from([]),
                127_886_946_205_696,
                Tombstone,
            ),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 2)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());

        assert_eq!(
            Some(items.get(1).cloned().unwrap()),
            data_block.point_read(&[233, 233])
        );
        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }

    #[test]
    fn data_block_point_read_tombstone() -> Result<()> {
        let items = [
            InternalValue::from_components("pla:saturn:fact", "Saturn is pretty big", 0, Value),
            InternalValue::from_components("pla:saturn:name", "Saturn", 0, Value),
            InternalValue::from_components("pla:venus:fact", "", 1, Tombstone),
            InternalValue::from_components("pla:venus:name", "Venus", 0, Value),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 16)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());

        assert!(
            data_block
                .point_read(b"pla:venus:fact")
                .expect("should exist")
                .is_tombstone()
        );

        Ok(())
    }

    #[test]
    fn data_block_point_read_dense_2() -> Result<()> {
        let items = [
            InternalValue::from_components("pla:earth:fact", "eaaaaaaaaarth", 0, Value),
            InternalValue::from_components("pla:jupiter:fact", "Jupiter is big", 0, Value),
            InternalValue::from_components("pla:jupiter:mass", "Massive", 0, Value),
            InternalValue::from_components("pla:jupiter:name", "Jupiter", 0, Value),
            InternalValue::from_components("pla:jupiter:radius", "Big", 0, Value),
            InternalValue::from_components("pla:saturn:fact", "Saturn is pretty big", 0, Value),
            InternalValue::from_components("pla:saturn:name", "Saturn", 0, Value),
            InternalValue::from_components("pla:venus:fact", "", 1, Tombstone),
            InternalValue::from_components("pla:venus:name", "Venus", 0, Value),
        ];

        let bytes = DataBlock::encode_into_vec(&items, 1)?;

        let data_block = DataBlock::new(Block {
            data: bytes.into(),
            header: Header {
                block_type: BlockType::Data,
                data_length: 0,
                uncompressed_length: 0,
            },
        });

        assert_eq!(data_block.len(), items.len());

        for needle in items {
            assert_eq!(
                Some(needle.clone()),
                data_block.point_read(&needle.key.user_key),
            );
        }

        assert_eq!(None, data_block.point_read(b"yyy"));

        Ok(())
    }
}
