use std::fs::File;

use byteorder::{LittleEndian, ReadBytesExt};

use super::{Block, BlockHandle, DataBlock};
use crate::{
    CompressionType, Error, KeyRange, Result, Slice, coding::Decode, table::block::BlockType,
};

/// Metadata required to read and compact a table.
#[derive(Debug)]
pub struct ParsedMeta {
    pub id: u32,
    pub data_block_count: u64,
    pub key_range: KeyRange,
    pub highest_seqno: u64,
    pub file_size: u64,
    pub item_count: u64,
    pub data_block_compression: CompressionType,
    pub index_block_compression: CompressionType,
}

impl ParsedMeta {
    pub fn load_with_handle(file: &File, handle: &BlockHandle) -> Result<Self> {
        let block = Block::from_file(file, *handle, CompressionType::None)?;
        if block.header.block_type != BlockType::Meta {
            return Err(Error::InvalidTag((
                "BlockType",
                block.header.block_type.into(),
            )));
        }
        let block = DataBlock::new(block);

        Ok(Self {
            id: Self::read_u32(&block, b"table_id")?,
            data_block_count: Self::read_u64(&block, b"block_count#data")?,
            key_range: KeyRange::new((
                Self::read(&block, b"key#min"),
                Self::read(&block, b"key#max"),
            )),
            highest_seqno: Self::read_u64(&block, b"seqno#max")?,
            file_size: Self::read_u64(&block, b"file_size")?,
            item_count: Self::read_u64(&block, b"item_count")?,
            data_block_compression: Self::read_compression(&block, b"compression#data")?,
            index_block_compression: Self::read_compression(&block, b"compression#index")?,
        })
    }

    fn read(block: &DataBlock, name: &[u8]) -> Slice {
        block
            .point_read(name)
            .unwrap_or_else(|| panic!("meta property {name:?} should exist"))
            .value
    }

    fn read_u32(block: &DataBlock, name: &[u8]) -> Result<u32> {
        Ok(Self::read(block, name)
            .as_ref()
            .read_u32::<LittleEndian>()?)
    }

    fn read_u64(block: &DataBlock, name: &[u8]) -> Result<u64> {
        Ok(Self::read(block, name)
            .as_ref()
            .read_u64::<LittleEndian>()?)
    }

    fn read_compression(block: &DataBlock, name: &[u8]) -> Result<CompressionType> {
        CompressionType::decode_from(&mut Self::read(block, name).as_ref())
    }
}
