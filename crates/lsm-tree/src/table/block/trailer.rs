// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::mem;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{
    Block,
    encoder::{Encodable, Encoder},
};
use crate::{Error, Result};

pub const TRAILER_START_MARKER: u8 = 255;

const TRAILER_SIZE: usize = 3 * mem::size_of::<u32>()
    + (2 * mem::size_of::<u8>())
    // Fixed key size
    + mem::size_of::<u8>()
    + mem::size_of::<u16>()
    // Fixed value size
    + mem::size_of::<u8>()
    + mem::size_of::<u32>();

/// Block trailer
///
/// ## Format
///
/// \[restart_interval\] \[binary index\]
/// \[fixed key length\] \[fixed value length\] \[item count\]
#[expect(clippy::doc_markdown)]
pub struct Trailer<'a> {
    block: &'a Block,
}

impl<'a> Trailer<'a> {
    pub fn new(block: &'a Block) -> Self {
        Self { block }
    }

    /// Returns the trailer position.
    pub fn trailer_offset(&self) -> usize {
        self.block.data.len() - TRAILER_SIZE
    }

    /// Returns the number of items in the block
    #[must_use]
    pub fn item_count(&self) -> usize {
        let reader = self.as_slice();

        #[expect(
            clippy::indexing_slicing,
            reason = "the item_count is at the end and is a u32"
        )]
        let reader = &mut &reader[(TRAILER_SIZE - mem::size_of::<u32>())..];

        #[expect(
            clippy::expect_used,
            reason = "the trailer offset is valid, and the trailer has a fixed size so the next item must be the item count"
        )]
        {
            reader
                .read_u32::<LittleEndian>()
                .expect("should read item count") as usize
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        #[expect(
            clippy::indexing_slicing,
            reason = "loaded blocks are validated to hold a trailer"
        )]
        &self.block.data[self.trailer_offset()..]
    }

    /// Checks the trailer and binary index of a data, index or meta block read
    /// from disk: decoders then trust its restart pointers.
    ///
    /// Layout: \[items\] \[marker\] \[binary index\] \[trailer\]
    pub fn validate(data: &[u8]) -> Result<()> {
        let trailer_offset = data
            .len()
            .checked_sub(TRAILER_SIZE)
            .ok_or(Error::InvalidTrailer)?;
        #[expect(clippy::indexing_slicing, reason = "trailer_offset <= data.len()")]
        let mut reader = &data[trailer_offset..];
        let restart_interval = reader.read_u8()?;
        let step_size = reader.read_u8()?;
        let len = reader.read_u32::<LittleEndian>()? as usize;
        let index_offset = reader.read_u32::<LittleEndian>()? as usize;

        let valid_layout = restart_interval > 0
            && (step_size == 2 || step_size == 4)
            && index_offset > 0
            && len
                .checked_mul(usize::from(step_size))
                .and_then(|size| size.checked_add(index_offset))
                == Some(trailer_offset)
            && data.get(index_offset - 1) == Some(&TRAILER_START_MARKER);
        if !valid_layout {
            return Err(Error::InvalidTrailer);
        }

        // Restart heads must start inside the item region (before the marker).
        let items_end = index_offset - 1;
        #[expect(
            clippy::indexing_slicing,
            reason = "the index spans index_offset..trailer_offset"
        )]
        let index = &data[index_offset..trailer_offset];
        let in_bounds = if step_size == 2 {
            index
                .as_chunks::<2>()
                .0
                .iter()
                .all(|ptr| usize::from(u16::from_le_bytes(*ptr)) < items_end)
        } else {
            index
                .as_chunks::<4>()
                .0
                .iter()
                .all(|ptr| (u32::from_le_bytes(*ptr) as usize) < items_end)
        };
        if !in_bounds {
            return Err(Error::InvalidTrailer);
        }

        Ok(())
    }

    pub fn write<S: Default, T: Encodable<S>>(encoder: Encoder<'_, S, T>) -> Result<()> {
        let Encoder {
            writer,
            items,
            restart_interval,
            binary_index_builder,
            fixed_key_len,
            fixed_value_len,
            ..
        } = encoder;

        // IMPORTANT: Terminator marker
        writer.write_u8(TRAILER_START_MARKER)?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "data blocks will never even approach 4 GB in size"
        )]
        let binary_index_offset = writer.len() as u32;

        // Write binary index
        let (binary_index_step_size, binary_index_len) = binary_index_builder.write(writer)?;

        // Write trailer

        #[cfg(debug_assertions)]
        let bytes_before = writer.len();

        writer.write_u8(restart_interval)?;

        writer.write_u8(binary_index_step_size)?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "even with a dense index, there can't be more index pointers than items"
        )]
        writer.write_u32::<LittleEndian>(binary_index_len as u32)?;

        writer.write_u32::<LittleEndian>(binary_index_offset)?;

        // Fixed key size
        writer.write_u8(u8::from(fixed_key_len.is_some()))?;
        writer.write_u16::<LittleEndian>(fixed_key_len.unwrap_or_default())?;

        // Fixed value size
        writer.write_u8(u8::from(fixed_value_len.is_some()))?;
        writer.write_u32::<LittleEndian>(fixed_value_len.unwrap_or_default())?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "data blocks will never even approach 4 GiB in size, so there can't be that many items either"
        )]
        writer.write_u32::<LittleEndian>(items.len() as u32)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            TRAILER_SIZE,
            writer.len() - bytes_before,
            "trailer size does not match",
        );

        Ok(())
    }
}
