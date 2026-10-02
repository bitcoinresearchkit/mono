#[cfg(feature = "storage")]
use std::mem;
#[cfg(feature = "storage")]
use vecdb::Error;
#[cfg(feature = "storage")]
use vecdb::Result;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable, Pco};

pub const OP_RETURN_KIND_COUNT: usize = OpReturnKind::Unknown as usize + 1;

#[derive(
    Debug,
    Clone,
    Copy,
    AsRefStr,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum OpReturnKind {
    Runes,
    VeriBlock,
    Omni,
    Stacks,
    Blockstack,
    Colu,
    OpenAssets,
    Komodo,
    CoinSpark,
    Poet,
    Docproof,
    OpenTimestamps,
    Factom,
    EternityWall,
    Memo,
    Bitproof,
    Ascribe,
    Stampery,
    Epobc,
    BareHash,
    Text,
    Empty,
    Unknown,
}

const OP_RETURN_KINDS: [OpReturnKind; OP_RETURN_KIND_COUNT] = [
    OpReturnKind::Runes,
    OpReturnKind::VeriBlock,
    OpReturnKind::Omni,
    OpReturnKind::Stacks,
    OpReturnKind::Blockstack,
    OpReturnKind::Colu,
    OpReturnKind::OpenAssets,
    OpReturnKind::Komodo,
    OpReturnKind::CoinSpark,
    OpReturnKind::Poet,
    OpReturnKind::Docproof,
    OpReturnKind::OpenTimestamps,
    OpReturnKind::Factom,
    OpReturnKind::EternityWall,
    OpReturnKind::Memo,
    OpReturnKind::Bitproof,
    OpReturnKind::Ascribe,
    OpReturnKind::Stampery,
    OpReturnKind::Epobc,
    OpReturnKind::BareHash,
    OpReturnKind::Text,
    OpReturnKind::Empty,
    OpReturnKind::Unknown,
];

impl OpReturnKind {
    #[cfg(feature = "storage")]
    fn is_valid(value: u8) -> bool {
        value <= Self::Unknown as u8
    }
}

#[cfg(feature = "storage")]
impl Formattable for OpReturnKind {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_ref().as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

#[cfg(feature = "storage")]
impl Bytes for OpReturnKind {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        }
        let value = bytes[0];
        if !Self::is_valid(value) {
            return Err(Error::InvalidArgument("invalid OpReturnKind"));
        }
        // SAFETY: We validated that value is a valid variant.
        Ok(unsafe { mem::transmute::<u8, Self>(value) })
    }
}

// SAFETY: The non-transparent conversion validates every decoded discriminant.
#[cfg(feature = "storage")]
unsafe impl Pco for OpReturnKind {
    type NumberType = u8;

    #[inline(always)]
    fn to_number(self) -> Self::NumberType {
        self as u8
    }

    #[inline(always)]
    fn from_number(value: Self::NumberType) -> Result<Self> {
        Self::from_bytes(&[value])
    }
}

impl OpReturnKind {
    pub const ALL: &'static [Self] = &OP_RETURN_KINDS;

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn get<T>(self, values: &[T; OP_RETURN_KIND_COUNT]) -> &T {
        &values[self.index()]
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "storage")]
    use super::*;

    #[cfg(feature = "storage")]
    #[test]
    fn pco_conversion_rejects_invalid_discriminants() {
        const { assert!(!OpReturnKind::IS_TRANSPARENT) };
        assert_eq!(
            OpReturnKind::from_number(OpReturnKind::Unknown as u8).unwrap(),
            OpReturnKind::Unknown
        );
        assert!(OpReturnKind::from_number(u8::MAX).is_err());
    }
}
