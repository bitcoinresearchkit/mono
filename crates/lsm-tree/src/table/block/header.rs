// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    io::{Read, Write},
    mem,
};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use crate::{
    Error, Result,
    coding::{Decode, Encode},
    file::MAGIC_BYTES,
    table::block::BlockType,
};

/// Header of a disk-based block.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub block_type: BlockType,

    /// On-disk size of data segment.
    pub data_length: u32,

    /// Uncompressed size of data segment.
    pub uncompressed_length: u32,
}

impl Header {
    #[must_use]
    pub const fn serialized_len() -> usize {
        MAGIC_BYTES.len()
            + mem::size_of::<BlockType>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
    }
}

impl Encode for Header {
    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&MAGIC_BYTES)?;
        writer.write_u8(self.block_type.into())?;
        writer.write_u32::<LE>(self.data_length)?;
        writer.write_u32::<LE>(self.uncompressed_length)?;
        Ok(())
    }
}

impl Decode for Header {
    fn decode_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0_u8; MAGIC_BYTES.len()];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC_BYTES {
            return Err(Error::InvalidHeader("Block"));
        }

        Ok(Self {
            block_type: BlockType::try_from(reader.read_u8()?)?,
            data_length: reader.read_u32::<LE>()?,
            uncompressed_length: reader.read_u32::<LE>()?,
        })
    }
}
