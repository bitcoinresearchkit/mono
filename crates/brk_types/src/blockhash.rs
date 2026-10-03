use std::{
    array::from_fn,
    fmt, mem,
    str::{self, FromStr},
};

use bitcoin::{BlockHash as BitcoinBlockHash, hashes::Hash};
use brk_error::Error;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Double-SHA256 block-header hash, serialized in Bitcoin's conventional
/// hexadecimal byte order.
#[derive(Default, Debug, Deref, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Bytes))]
#[repr(C)]
#[schemars(
    transparent,
    with = "String",
    example = &"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
    example = &"0000000000000000000320283a032748cef8227873ff4872689bf23f1cda83a5"
)]
pub struct BlockHash([u8; 32]);

impl BlockHash {
    fn hex(&self) -> [u8; 64] {
        from_fn(|i| {
            let byte = self.0[31 - i / 2];
            let nibble = if i % 2 == 0 { byte >> 4 } else { byte & 15 };
            b"0123456789abcdef"[nibble as usize]
        })
    }
}

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

impl From<BitcoinBlockHash> for BlockHash {
    #[inline]
    fn from(value: BitcoinBlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BitcoinBlockHash> for &BlockHash {
    #[inline]
    fn from(value: &BitcoinBlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<BlockHash> for BitcoinBlockHash {
    #[inline]
    fn from(value: BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BlockHash> for &BitcoinBlockHash {
    #[inline]
    fn from(value: &BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BlockHash> for BitcoinBlockHash {
    #[inline]
    fn from(value: &BlockHash) -> Self {
        BitcoinBlockHash::from_slice(&value.0).unwrap()
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(str::from_utf8(&self.hex()).unwrap())
    }
}

impl Serialize for BlockHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(str::from_utf8(&self.hex()).unwrap())
    }
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(feature = "storage")]
impl Formattable for BlockHash {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.hex());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
