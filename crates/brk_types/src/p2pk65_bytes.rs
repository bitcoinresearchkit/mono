use std::fmt;

use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::U8x65;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// An uncompressed (65-byte) public key.
#[derive(Debug, Clone, Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct P2PK65Bytes(#[cfg_attr(feature = "schemars", schemars(with = "String"))] U8x65);

impl From<&[u8]> for P2PK65Bytes {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(U8x65::from(value))
    }
}

impl From<U8x65> for P2PK65Bytes {
    #[inline]
    fn from(value: U8x65) -> Self {
        Self(value)
    }
}

/// Lowercase hex.
impl fmt::Display for P2PK65Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}

impl Serialize for P2PK65Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for P2PK65Bytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let hex = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        crate::bytes::parse_hex::<65>(&hex)
            .map(|bytes| Self::from(&bytes[..]))
            .ok_or_else(|| D::Error::custom("expected 65 bytes of hex"))
    }
}

#[cfg(feature = "storage")]
impl Formattable for P2PK65Bytes {
    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        crate::bytes::push_hex(&self.0[..], buf);
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
