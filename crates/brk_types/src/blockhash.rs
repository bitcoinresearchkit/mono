use std::{fmt, mem, str::FromStr};

use bitcoin::hashes::Hash;
use brk_error::Error;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer, de};
use vecdb::{Bytes, Formattable};

/// Double-SHA256 block-header hash, serialized in Bitcoin's conventional
/// hexadecimal byte order.
#[derive(Default, Debug, Deref, Clone, Copy, PartialEq, Eq, Hash, Bytes, JsonSchema)]
#[repr(C)]
#[schemars(
    transparent,
    with = "String",
    example = &"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
    example = &"0000000000000000000320283a032748cef8227873ff4872689bf23f1cda83a5"
)]
pub struct BlockHash([u8; 32]);

impl TryFrom<&str> for BlockHash {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(bitcoin::BlockHash::from_str(s)?))
    }
}

impl FromStr for BlockHash {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl From<bitcoin::BlockHash> for BlockHash {
    #[inline]
    fn from(value: bitcoin::BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&bitcoin::BlockHash> for &BlockHash {
    #[inline]
    fn from(value: &bitcoin::BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<BlockHash> for bitcoin::BlockHash {
    #[inline]
    fn from(value: BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BlockHash> for &bitcoin::BlockHash {
    #[inline]
    fn from(value: &BlockHash) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BlockHash> for bitcoin::BlockHash {
    #[inline]
    fn from(value: &BlockHash) -> Self {
        bitcoin::BlockHash::from_slice(&value.0).unwrap()
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&bitcoin::BlockHash::from(self).to_string())
    }
}

impl Serialize for BlockHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

impl Formattable for BlockHash {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
