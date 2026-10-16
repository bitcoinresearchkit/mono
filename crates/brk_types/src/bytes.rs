use std::fmt;

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Appends `bytes` as lowercase hex.
#[inline]
pub(crate) fn push_hex(bytes: &[u8], buf: &mut Vec<u8>) {
    buf.reserve(bytes.len() * 2);
    for &byte in bytes {
        buf.push(HEX[usize::from(byte >> 4)]);
        buf.push(HEX[usize::from(byte & 0x0f)]);
    }
}

/// A 32-byte hash as lowercase hex in reversed byte order (Bitcoin's display order for txids
/// and block hashes).
#[inline]
pub(crate) fn reversed_hex(bytes: &[u8; 32]) -> [u8; 64] {
    std::array::from_fn(|i| {
        let byte = bytes[31 - i / 2];
        HEX[usize::from(if i % 2 == 0 { byte >> 4 } else { byte & 0x0f })]
    })
}

/// Writes `bytes` as lowercase hex.
pub(crate) fn fmt_hex(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use fmt::Write;
    bytes.iter().try_for_each(|&byte| {
        f.write_char(char::from(HEX[usize::from(byte >> 4)]))?;
        f.write_char(char::from(HEX[usize::from(byte & 0x0f)]))
    })
}

/// Parses exactly `N` bytes of hex (either case).
pub(crate) fn parse_hex<const N: usize>(hex: &str) -> Option<[u8; N]> {
    let digits = hex.as_bytes();
    if digits.len() != N * 2 {
        return None;
    }
    let nibble = |digit: u8| (digit as char).to_digit(16).map(|value| value as u8);
    let mut bytes = [0; N];
    let (pairs, _) = digits.as_chunks::<2>();
    for (byte, &[high, low]) in bytes.iter_mut().zip(pairs) {
        *byte = (nibble(high)? << 4) | nibble(low)?;
    }
    Some(bytes)
}

/// Serializes a `Display` value as a string without building one (scripts: `Display` is assembly,
/// `format_args!("{:x}")` is hex).
pub(crate) struct DisplayStr<T>(pub T);

impl<T: fmt::Display> serde::Serialize for DisplayStr<T> {
    #[inline]
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

/// The impls a 32-byte hash newtype (`struct $t([u8; 32])`) shares with its rust-bitcoin
/// counterpart `$bitcoin`: owned and zero-cost reference conversions both ways, and Bitcoin's
/// reversed-hex text for `Display`, serde (parsed by the type's `FromStr`) and vector output.
macro_rules! impl_hash32 {
    ($t:ident, $bitcoin:ty, expecting: $expecting:literal) => {
        // The reference casts between the two types rely on this. A rust-bitcoin hash wraps
        // `sha256d::Hash` (`repr(transparent)` over `[u8; 32]`) with the default representation,
        // which guarantees no layout by itself; but at 32 bytes and alignment 1 its one 32-byte
        // field can only sit at offset 0 with no padding. Both types then are 32 bytes valid for
        // every bit pattern.
        const _: () = assert!(
            size_of::<$t>() == 32
                && size_of::<$bitcoin>() == 32
                && align_of::<$t>() == 1
                && align_of::<$bitcoin>() == 1
        );

        impl $t {
            #[inline]
            fn hex(&self) -> [u8; 64] {
                $crate::bytes::reversed_hex(&self.0)
            }
        }

        impl From<$bitcoin> for $t {
            #[inline]
            fn from(value: $bitcoin) -> Self {
                Self(::bitcoin::hashes::Hash::to_byte_array(value))
            }
        }

        impl From<&$bitcoin> for &$t {
            #[inline]
            fn from(value: &$bitcoin) -> Self {
                // SAFETY: same layout (see the layout assertion).
                unsafe { ::std::mem::transmute(value) }
            }
        }

        impl From<$t> for $bitcoin {
            #[inline]
            fn from(value: $t) -> Self {
                ::bitcoin::hashes::Hash::from_byte_array(value.0)
            }
        }

        impl From<&$t> for $bitcoin {
            #[inline]
            fn from(value: &$t) -> Self {
                ::bitcoin::hashes::Hash::from_byte_array(value.0)
            }
        }

        impl From<&$t> for &$bitcoin {
            #[inline]
            fn from(value: &$t) -> Self {
                // SAFETY: same layout (see the layout assertion).
                unsafe { ::std::mem::transmute(value) }
            }
        }

        impl ::std::fmt::Display for $t {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(::std::str::from_utf8(&self.hex()).unwrap())
            }
        }

        impl ::serde::Serialize for $t {
            fn serialize<S: ::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(::std::str::from_utf8(&self.hex()).unwrap())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $t {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::std::result::Result<Self, D::Error> {
                struct Visitor;

                impl ::serde::de::Visitor<'_> for Visitor {
                    type Value = $t;

                    fn expecting(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        f.write_str($expecting)
                    }

                    fn visit_str<E: ::serde::de::Error>(
                        self,
                        value: &str,
                    ) -> ::std::result::Result<$t, E> {
                        value.parse().map_err(E::custom)
                    }
                }

                deserializer.deserialize_str(Visitor)
            }
        }

        #[cfg(feature = "storage")]
        impl ::vecdb::Formattable for $t {
            fn write_to(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.hex());
            }

            fn fmt_json(&self, buf: &mut Vec<u8>) {
                buf.push(b'"');
                self.write_to(buf);
                buf.push(b'"');
            }
        }
    };
}

pub(crate) use impl_hash32;
