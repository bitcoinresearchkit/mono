//! The script payloads an address type indexes: fixed-size bytes, lowercase hex in JSON.

use crate::{U8x2, U8x20, U8x32, U8x33, U8x65};

macro_rules! payload_bytes {
    ($(#[$meta:meta])* $name:ident($inner:ident, $len:literal)) => {
        $(#[$meta])*
        #[derive(Debug, Clone, ::derive_more::Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "storage", derive(::vecdb::Bytes))]
        pub struct $name(#[cfg_attr(feature = "schemars", schemars(with = "String"))] $inner);

        impl From<&[u8]> for $name {
            #[inline]
            fn from(value: &[u8]) -> Self {
                Self($inner::from(value))
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        /// Lowercase hex.
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                crate::bytes::fmt_hex(&self.0[..], f)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let hex = <::std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
                crate::bytes::parse_hex::<$len>(&hex)
                    .map(|bytes| Self::from(&bytes[..]))
                    .ok_or_else(|| {
                        <D::Error as ::serde::de::Error>::custom(concat!(
                            "expected ",
                            stringify!($len),
                            " bytes of hex"
                        ))
                    })
            }
        }

        #[cfg(feature = "storage")]
        impl ::vecdb::Formattable for $name {
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
    };
}

payload_bytes! {
    /// The 2-byte witness program of a P2A (pay-to-anchor) output.
    P2ABytes(U8x2, 2)
}

payload_bytes! {
    /// A compressed (33-byte) public key.
    P2PK33Bytes(U8x33, 33)
}

payload_bytes! {
    /// An uncompressed (65-byte) public key.
    P2PK65Bytes(U8x65, 65)
}

payload_bytes! {
    /// The 20-byte public key hash of a P2PKH output.
    P2PKHBytes(U8x20, 20)
}

payload_bytes! {
    /// The 20-byte script hash of a P2SH output.
    P2SHBytes(U8x20, 20)
}

payload_bytes! {
    /// The 32-byte output key of a P2TR output.
    P2TRBytes(U8x32, 32)
}

payload_bytes! {
    /// The 20-byte public key hash of a P2WPKH output.
    P2WPKHBytes(U8x20, 20)
}

payload_bytes! {
    /// The 32-byte script hash of a P2WSH output.
    P2WSHBytes(U8x32, 32)
}
