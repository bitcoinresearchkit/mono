// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fmt::{Display, Formatter, Result},
    io::{Read, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{
    Error, Result as CrateResult,
    coding::{Decode, Encode},
};

/// Compression algorithm to use
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CompressionType {
    /// No compression
    ///
    /// Not recommended.
    None,

    /// LZ4 compression
    ///
    /// Recommended for use cases with a focus
    /// on speed over compression ratio.
    Lz4,
}

impl Display for CompressionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",

                Self::Lz4 => "lz4",
            }
        )
    }
}

impl Encode for CompressionType {
    fn encode_into<W: Write>(&self, writer: &mut W) -> CrateResult<()> {
        match self {
            Self::None => {
                writer.write_u8(0)?;
            }

            Self::Lz4 => {
                writer.write_u8(1)?;
            }
        }

        Ok(())
    }
}

impl Decode for CompressionType {
    fn decode_from<R: Read>(reader: &mut R) -> CrateResult<Self> {
        let tag = reader.read_u8()?;

        match tag {
            0 => Ok(Self::None),

            1 => Ok(Self::Lz4),

            tag => Err(Error::InvalidTag(("CompressionType", tag))),
        }
    }
}
