use std::{
    fs::File,
    io::{Read, Write},
};

use log::trace;
use lz4_flex::{compress, decompress_into};

use crate::{
    CompressionType, Error, Result, Slice,
    coding::{Decode, Encode},
    file,
    table::BlockHandle,
};

// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

pub mod binary_index;
pub mod decoder;
mod encoder;
mod header;
mod offset;
mod trailer;
mod r#type;

pub use decoder::{Decodable, Decoder, ParsedItem};
pub use encoder::{Encodable, Encoder};
pub use header::Header;
pub use offset::BlockOffset;
pub use trailer::{TRAILER_START_MARKER, Trailer};
pub use r#type::BlockType;

const MAX_LZ4_EXPANSION_RATIO: usize = 256;

fn validate_lengths(header: &Header, compression: CompressionType) -> Result<()> {
    let data_length =
        usize::try_from(header.data_length).map_err(|_| Error::InvalidHeader("Block length"))?;
    let uncompressed_length = usize::try_from(header.uncompressed_length)
        .map_err(|_| Error::InvalidHeader("Block length"))?;

    let valid = match compression {
        CompressionType::None => data_length == uncompressed_length,
        CompressionType::Lz4 => {
            uncompressed_length <= data_length.saturating_mul(MAX_LZ4_EXPANSION_RATIO)
        }
    };

    if !valid {
        return Err(Error::InvalidHeader("Block length"));
    }

    Ok(())
}

fn decompress_lz4(raw_data: &[u8], uncompressed_len: usize) -> Result<Slice> {
    let mut builder = Slice::builder(uncompressed_len);

    let written = decompress_into(raw_data, &mut builder)
        .map_err(|_| Error::Decompress(CompressionType::Lz4))?;

    if written != builder.len() {
        return Err(Error::Decompress(CompressionType::Lz4));
    }

    Ok(builder.freeze().into())
}

/// A block on disk
///
/// Consists of a fixed-size header and some bytes (the data/payload).
#[derive(Clone)]
pub struct Block {
    pub header: Header,
    pub data: Slice,
}

impl Block {
    /// Returns the uncompressed block size in bytes.
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Encodes a block into a writer.
    pub fn write_into<W: Write>(
        mut writer: &mut W,
        data: &[u8],
        block_type: BlockType,
        compression: CompressionType,
    ) -> Result<Header> {
        let mut header = Header {
            block_type,
            data_length: 0, // <-- NOTE: Is set later on

            #[expect(clippy::cast_possible_truncation, reason = "blocks are limited to u32")]
            uncompressed_length: data.len() as u32,
        };

        let data = match compression {
            CompressionType::None => data,

            CompressionType::Lz4 => &compress(data),
        };

        #[expect(clippy::cast_possible_truncation, reason = "blocks are limited to u32")]
        {
            header.data_length = data.len() as u32;
        }

        header.encode_into(&mut writer)?;
        writer.write_all(data)?;

        trace!(
            "Writing block with size {}B (compressed: {}B) (excluding header of {}B)",
            header.uncompressed_length,
            header.data_length,
            Header::serialized_len(),
        );

        Ok(header)
    }

    /// Reads a block from a reader.
    pub fn from_reader<R: Read>(reader: &mut R, compression: CompressionType) -> Result<Self> {
        let header = Header::decode_from(reader)?;
        validate_lengths(&header, compression)?;
        let raw_data = Slice::from_reader(reader, header.data_length as usize)?;

        let data = match compression {
            CompressionType::None => raw_data,

            CompressionType::Lz4 => decompress_lz4(&raw_data, header.uncompressed_length as usize)?,
        };

        debug_assert_eq!(header.uncompressed_length, {
            #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
            {
                data.len() as u32
            }
        });

        Ok(Self { header, data })
    }

    /// Reads a block from a file.
    pub fn from_file(
        file: &File,
        handle: BlockHandle,
        compression: CompressionType,
    ) -> Result<Self> {
        let buf = file::read_exact(file, *handle.offset(), handle.size() as usize)?;

        let header = Header::decode_from(&mut &buf[..])?;
        validate_lengths(&header, compression)?;

        let expected_length = Header::serialized_len()
            .checked_add(header.data_length as usize)
            .ok_or(Error::InvalidHeader("Block length"))?;
        if expected_length != buf.len() {
            return Err(Error::InvalidHeader("Block length"));
        }

        let buf = match compression {
            CompressionType::None => {
                let value = buf.slice(Header::serialized_len()..);

                #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
                {
                    debug_assert_eq!(header.uncompressed_length, value.len() as u32);
                }

                value
            }

            CompressionType::Lz4 => {
                // NOTE: We know that a header always exists and data is never empty
                // So the slice is fine
                #[expect(clippy::indexing_slicing)]
                let raw_data = &buf[Header::serialized_len()..];

                decompress_lz4(raw_data, header.uncompressed_length as usize)?
            }
        };

        Ok(Self { header, data: buf })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use test_log::test;

    use super::*;
    use crate::coding::Encode;

    // TODO: Block::from_file roundtrips

    #[test]
    fn block_roundtrip_uncompressed() -> Result<()> {
        let mut writer = vec![];

        Block::write_into(
            &mut writer,
            b"abcdefabcdefabcdef",
            BlockType::Data,
            CompressionType::None,
        )?;

        {
            let mut reader = &writer[..];
            let block = Block::from_reader(&mut reader, CompressionType::None)?;
            assert_eq!(b"abcdefabcdefabcdef", &*block.data);
        }

        Ok(())
    }
    #[test]
    fn block_roundtrip_lz4() -> Result<()> {
        let mut writer = vec![];

        Block::write_into(
            &mut writer,
            b"abcdefabcdefabcdef",
            BlockType::Data,
            CompressionType::Lz4,
        )?;

        {
            let mut reader = &writer[..];
            let block = Block::from_reader(&mut reader, CompressionType::Lz4)?;
            assert_eq!(b"abcdefabcdefabcdef", &*block.data);
        }

        Ok(())
    }

    #[test]
    fn block_from_file_rejects_handle_size_mismatch() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("block");
        let mut bytes = Vec::new();
        Block::write_into(
            &mut bytes,
            b"abcdef",
            BlockType::Data,
            CompressionType::None,
        )?;
        bytes.push(0);
        fs::write(&path, &bytes)?;

        let handle = BlockHandle::new(BlockOffset(0), bytes.len() as u32);
        assert!(matches!(
            Block::from_file(&File::open(path)?, handle, CompressionType::None),
            Err(Error::InvalidHeader("Block length"))
        ));
        Ok(())
    }

    #[test]
    fn block_rejects_invalid_uncompressed_length() {
        let header = Header {
            block_type: BlockType::Data,
            data_length: 1,
            uncompressed_length: 2,
        };
        let mut bytes = header.encode_into_vec();
        bytes.push(0);

        assert!(matches!(
            Block::from_reader(&mut &bytes[..], CompressionType::None),
            Err(Error::InvalidHeader("Block length"))
        ));
    }

    #[test]
    fn block_rejects_impossible_lz4_expansion_before_allocating() {
        let header = Header {
            block_type: BlockType::Data,
            data_length: 1,
            uncompressed_length: 257,
        };
        let mut bytes = header.encode_into_vec();
        bytes.push(0);

        assert!(matches!(
            Block::from_reader(&mut &bytes[..], CompressionType::Lz4),
            Err(Error::InvalidHeader("Block length"))
        ));
    }
}
