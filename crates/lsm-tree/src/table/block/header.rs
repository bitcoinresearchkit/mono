// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::coding::{Decode, Encode};
use crate::file::MAGIC_BYTES;
use crate::table::block::BlockType;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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
            + std::mem::size_of::<BlockType>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u32>()
    }
}

impl Encode for Header {
    fn encode_into<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_all(&MAGIC_BYTES)?;
        writer.write_u8(self.block_type.into())?;
        writer.write_u32::<byteorder::LE>(self.data_length)?;
        writer.write_u32::<byteorder::LE>(self.uncompressed_length)?;
        Ok(())
    }
}

impl Decode for Header {
    fn decode_from<R: Read>(reader: &mut R) -> crate::Result<Self> {
        let mut magic = [0_u8; MAGIC_BYTES.len()];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC_BYTES {
            return Err(crate::Error::InvalidHeader("Block"));
        }

        Ok(Self {
            block_type: BlockType::try_from(reader.read_u8()?)?,
            data_length: reader.read_u32::<byteorder::LE>()?,
            uncompressed_length: reader.read_u32::<byteorder::LE>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn block_header_serde_roundtrip() -> crate::Result<()> {
        let header = Header {
            block_type: BlockType::Data,
            data_length: 252_356,
            uncompressed_length: 124_124_124,
        };

        let bytes = header.encode_into_vec();
        assert_eq!(bytes.len(), Header::serialized_len());
        assert_eq!(header, Header::decode_from(&mut &bytes[..])?);
        Ok(())
    }

    #[test]
    fn block_header_rejects_invalid_magic() {
        let header = Header {
            block_type: BlockType::Data,
            data_length: 252_356,
            uncompressed_length: 124_124_124,
        };

        let mut bytes = header.encode_into_vec();
        bytes[0] ^= 1;
        assert!(matches!(
            Header::decode_from(&mut &bytes[..]),
            Err(crate::Error::InvalidHeader("Block"))
        ));
    }
}
