//! The script payloads an address type indexes: fixed-size bytes, lowercase hex in JSON.

use crate::OutputType;

macro_rules! payload_bytes {
    ($($(#[$meta:meta])* $output_type:ident => $name:ident($len:literal),)+) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, ::derive_more::Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
            #[cfg_attr(feature = "storage", derive(::vecdb::Bytes))]
            pub struct $name(#[cfg_attr(feature = "schemars", schemars(with = "String"))] [u8; $len]);

            /// Panics unless `value` holds exactly the payload's length.
            impl From<&[u8]> for $name {
                #[inline]
                fn from(value: &[u8]) -> Self {
                    Self(value.try_into().expect(concat!(stringify!($name), " takes ", stringify!($len), " bytes")))
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
                        .map(Self)
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
        )+

        /// Each address type with its payload length in bytes.
        pub const ADDRESS_PAYLOAD_LENGTHS: &[(OutputType, usize)] = &[$((OutputType::$output_type, $len)),+];

        impl OutputType {
            /// The payload length an address of this type indexes; `None` for types without addresses.
            pub const fn payload_len(self) -> Option<usize> {
                match self {
                    $(Self::$output_type => Some($len),)+
                    _ => None,
                }
            }
        }
    };
}

payload_bytes! {
    /// An uncompressed (65-byte) public key.
    P2PK65 => P2PK65Bytes(65),
    /// A compressed (33-byte) public key.
    P2PK33 => P2PK33Bytes(33),
    /// The 20-byte public key hash of a P2PKH output.
    P2PKH => P2PKHBytes(20),
    /// The 20-byte script hash of a P2SH output.
    P2SH => P2SHBytes(20),
    /// The 20-byte public key hash of a P2WPKH output.
    P2WPKH => P2WPKHBytes(20),
    /// The 32-byte script hash of a P2WSH output.
    P2WSH => P2WSHBytes(32),
    /// The 32-byte output key of a P2TR output.
    P2TR => P2TRBytes(32),
    /// The 2-byte witness program of a P2A (pay-to-anchor) output.
    P2A => P2ABytes(2),
}
