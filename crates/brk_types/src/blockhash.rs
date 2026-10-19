use std::str::FromStr;

use bitcoin::BlockHash as BitcoinBlockHash;
use brk_error::Error;
use derive_more::Deref;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[cfg(feature = "storage")]
use vecdb::Bytes;

/// Double-SHA256 block-header hash, serialized in Bitcoin's conventional
/// hexadecimal byte order.
#[derive(Default, Debug, Deref, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Bytes))]
#[repr(C)]
#[cfg_attr(feature = "schemars", schemars(
    transparent,
    with = "String",
    example = &"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
    example = &"0000000000000000000320283a032748cef8227873ff4872689bf23f1cda83a5"
))]
pub struct BlockHash([u8; 32]);

crate::bytes::impl_hash32!(BlockHash, BitcoinBlockHash, expecting: "a block hash");

impl TryFrom<&str> for BlockHash {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(BitcoinBlockHash::from_str(s)?))
    }
}

impl FromStr for BlockHash {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}
