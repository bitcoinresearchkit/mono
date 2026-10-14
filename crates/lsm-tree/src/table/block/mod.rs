use std::{
    fs::File,
    io::{Read, Write},
};

use log::trace;
use lz4_flex::block::{
    CompressTable, compress_into_with_table, decompress_into, get_maximum_output_size,
};

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

/// Output buffer and hash table reused across LZ4 block compressions.
#[derive(Default)]
pub struct Lz4Scratch {
    table: CompressTable,
    output: Vec<u8>,
}

impl Lz4Scratch {
    fn compress(&mut self, data: &[u8]) -> &[u8] {
        let max_len = get_maximum_output_size(data.len());
        if self.output.len() < max_len {
            self.output.resize(max_len, 0);
        }
        #[expect(
            clippy::expect_used,
            reason = "the output holds the maximum compressed size"
        )]
        let len = compress_into_with_table(data, &mut self.output, &mut self.table)
            .expect("output should hold the compressed block");
        #[expect(
            clippy::indexing_slicing,
            reason = "LZ4 writes at most the output length"
        )]
        &self.output[..len]
    }
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

    /// Writes an uncompressed block.
    pub fn write_into<W: Write>(
        writer: &mut W,
        data: &[u8],
        block_type: BlockType,
    ) -> Result<Header> {
        Self::write_raw(writer, data, data.len(), block_type)
    }

    /// Writes an LZ4-compressed block.
    pub fn write_lz4_into<W: Write>(
        writer: &mut W,
        data: &[u8],
        block_type: BlockType,
        scratch: &mut Lz4Scratch,
    ) -> Result<Header> {
        Self::write_raw(writer, scratch.compress(data), data.len(), block_type)
    }

    fn write_raw<W: Write>(
        mut writer: &mut W,
        data: &[u8],
        uncompressed_length: usize,
        block_type: BlockType,
    ) -> Result<Header> {
        #[expect(clippy::cast_possible_truncation, reason = "blocks are limited to u32")]
        let header = Header {
            block_type,
            data_length: data.len() as u32,
            uncompressed_length: uncompressed_length as u32,
        };

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
