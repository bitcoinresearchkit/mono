use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Deref, Clone, JsonSchema)]
pub struct CoinbaseTag(#[schemars(with = "String")] [u8; 101]);

impl Bytes for CoinbaseTag {
    type Array = [u8; 101];
    const IS_NATIVE_LAYOUT: bool = true;

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        self.0
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        let arr: [u8; 101] = bytes.try_into().map_err(|_| vecdb::Error::WrongLength {
            received: bytes.len(),
            expected: 101,
        })?;
        Ok(Self(arr))
    }
}

impl CoinbaseTag {
    /// Returns the tag as a string, decoding each byte as its latin-1/Unicode codepoint.
    #[inline]
    pub fn as_str(&self) -> String {
        let len = (self.0[0] as usize).min(100);
        self.0[1..1 + len].iter().map(|&b| b as char).collect()
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

impl Serialize for CoinbaseTag {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> Deserialize<'de> for CoinbaseTag {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from(s.as_bytes()))
    }
}

impl Formattable for CoinbaseTag {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_str().as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        for &b in self.as_str().as_bytes() {
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
                _ => buf.push(b),
            }
        }
        buf.push(b'"');
    }
}

#[cfg(test)]
mod tests {
    use super::CoinbaseTag;

    #[test]
    fn preserves_raw_bytes_as_codepoints_and_truncates_at_100() {
        let tag = CoinbaseTag::from([b'A', 0xff, 0x00].as_slice());
        assert_eq!(tag.as_str(), "A\u{ff}\0");

        let bytes = (0_u8..=100).collect::<Vec<_>>();
        let tag = CoinbaseTag::from(bytes.as_slice());
        assert_eq!(tag.as_str().chars().count(), 100);
        assert_eq!(tag.as_str().chars().last(), Some(char::from(99)));
    }
}
