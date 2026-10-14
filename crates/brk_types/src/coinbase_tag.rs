use std::fmt::{self, Display, Formatter, Write};

use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

#[cfg(feature = "storage")]
use vecdb::Error;
#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Up to the first 100 bytes of a coinbase transaction's first-input
/// `scriptSig`. Bytes are preserved for storage and exposed as a string by
/// mapping each byte to the same-valued Unicode code point. Pool attribution
/// may search this raw value, but the value itself is not a normalized pool
/// label.
///
/// Stored as a fixed 101-byte record (1 byte length + 100 bytes data).
/// Uses `[u8; 101]` internally so that `size_of::<CoinbaseTag>()` matches
/// the serialized `Bytes::Array` size (vecdb requires this for alignment).
///
/// Bitcoin consensus limits coinbase scriptSig to 2-100 bytes.
#[derive(Debug, Deref, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct CoinbaseTag(#[cfg_attr(feature = "schemars", schemars(with = "String"))] [u8; 101]);

#[cfg(feature = "storage")]
// SAFETY: a single `[u8; 101]` field, so its bytes are its in-memory bytes, valid for every bit
// pattern.
unsafe impl Bytes for CoinbaseTag {
    type Array = [u8; 101];
    const IS_NATIVE_LAYOUT: bool = true;

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        self.0
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> VecdbResult<Self> {
        let arr: [u8; 101] = bytes.try_into().map_err(|_| Error::WrongLength {
            received: bytes.len(),
            expected: 101,
        })?;
        Ok(Self(arr))
    }
}

impl CoinbaseTag {
    /// The raw tag bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let len = (self.0[0] as usize).min(100);
        &self.0[1..1 + len]
    }

    /// The tag as text: each byte is the Latin-1/Unicode code point of the same value.
    #[inline]
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.as_bytes().iter().map(|&b| char::from(b))
    }
}

impl From<&[u8]> for CoinbaseTag {
    #[inline]
    fn from(bytes: &[u8]) -> Self {
        let truncated = &bytes[..bytes.len().min(100)];
        let len = truncated.len() as u8;
        let mut out = [0u8; 101];
        out[0] = len;
        out[1..1 + len as usize].copy_from_slice(truncated);
        Self(out)
    }
}

impl Display for CoinbaseTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.chars().try_for_each(|c| f.write_char(c))
    }
}

impl Serialize for CoinbaseTag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// The inverse of [`CoinbaseTag::chars`]: a code point above U+00FF is not a tag byte.
impl<'de> Deserialize<'de> for CoinbaseTag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let bytes = s
            .chars()
            .map(|c| {
                u8::try_from(c).map_err(|_| D::Error::custom("coinbase tag char above U+00FF"))
            })
            .collect::<Result<Vec<u8>, _>>()?;
        Ok(Self::from(bytes.as_slice()))
    }
}

/// Writes `byte` as the UTF-8 encoding of the code point of the same value.
#[cfg(feature = "storage")]
#[inline(always)]
fn push_latin1(buf: &mut Vec<u8>, byte: u8) {
    if byte < 0x80 {
        buf.push(byte);
    } else {
        buf.extend_from_slice(&[0xC0 | (byte >> 6), 0x80 | (byte & 0x3F)]);
    }
}

#[cfg(feature = "storage")]
impl Formattable for CoinbaseTag {
    fn write_to(&self, buf: &mut Vec<u8>) {
        for &b in self.as_bytes() {
            push_latin1(buf, b);
        }
    }

    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        let start = buf.len();
        self.write_to(buf);
        vecdb::escape_csv_cell(buf, start);
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        for &b in self.as_bytes() {
            match b {
                b'"' => buf.extend_from_slice(b"\\\""),
                b'\\' => buf.extend_from_slice(b"\\\\"),
                b'\n' => buf.extend_from_slice(b"\\n"),
                b'\r' => buf.extend_from_slice(b"\\r"),
                b'\t' => buf.extend_from_slice(b"\\t"),
                0x00..=0x1f => {
                    buf.extend_from_slice(b"\\u00");
                    buf.push(b"0123456789abcdef"[(b >> 4) as usize]);
                    buf.push(b"0123456789abcdef"[(b & 0xf) as usize]);
                }
                _ => push_latin1(buf, b),
            }
        }
        buf.push(b'"');
    }
}
