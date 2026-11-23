use std::{fmt, io::Write, sync::LazyLock};

use bitcoin::{Opcode, Script, opcodes::all::OP_PUSHBYTES_0, script::Instruction};

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Appends `bytes` as lowercase hex.
#[inline]
pub fn push_hex(bytes: &[u8], buf: &mut Vec<u8>) {
    let start = buf.len();
    buf.resize(start + bytes.len() * 2, 0);
    write_hex(bytes, &mut buf[start..]);
}

/// Writes `bytes` as lowercase hex into `out`, exactly twice their length: in pairs into space
/// made once, as a `push` per digit checks the capacity each time.
#[inline]
fn write_hex(bytes: &[u8], out: &mut [u8]) {
    debug_assert_eq!(out.len(), bytes.len() * 2, "hex output is twice its input");
    for (pair, &byte) in out.as_chunks_mut::<2>().0.iter_mut().zip(bytes) {
        *pair = [HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]];
    }
}

/// A 32-byte hash as lowercase hex in reversed byte order (Bitcoin's display order for txids
/// and block hashes).
#[inline]
pub fn reversed_hex(bytes: &[u8; 32]) -> [u8; 64] {
    std::array::from_fn(|i| {
        let byte = bytes[31 - i / 2];
        HEX[usize::from(if i % 2 == 0 { byte >> 4 } else { byte & 0x0f })]
    })
}

/// Writes `bytes` as lowercase hex, a stack buffer at a time: each write costs a formatter call,
/// and inside a JSON string an escape scan too.
pub(crate) fn fmt_hex(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut buf = [0; 1024];
    bytes.chunks(buf.len() / 2).try_for_each(|chunk| {
        let hex = &mut buf[..chunk.len() * 2];
        write_hex(chunk, hex);
        f.write_str(std::str::from_utf8(hex).expect("hex digits are ASCII"))
    })
}

/// Each ASCII byte's hex digit value (either case), `NOT_HEX` for any other byte.
const NIBBLES: [u8; 256] = {
    let mut table = [NOT_HEX; 256];
    let mut value = 0;
    while value < 16 {
        table[HEX[value] as usize] = value as u8;
        table[HEX[value].to_ascii_uppercase() as usize] = value as u8;
        value += 1;
    }
    table
};
const NOT_HEX: u8 = u8::MAX;

#[inline]
fn nibble(digit: u8) -> Option<u8> {
    Some(NIBBLES[usize::from(digit)]).filter(|&value| value != NOT_HEX)
}

/// Parses exactly `N` bytes of hex (either case).
pub(crate) fn parse_hex<const N: usize>(hex: &str) -> Option<[u8; N]> {
    let digits = hex.as_bytes();
    if digits.len() != N * 2 {
        return None;
    }
    let mut bytes = [0; N];
    let (pairs, _) = digits.as_chunks::<2>();
    for (byte, &[high, low]) in bytes.iter_mut().zip(pairs) {
        *byte = (nibble(high)? << 4) | nibble(low)?;
    }
    Some(bytes)
}

/// [`reversed_hex`]'s inverse: 64 hex digits (either case) as a hash in reversed byte order.
#[inline]
pub(crate) fn parse_reversed_hex(digits: &[u8; 64]) -> Option<[u8; 32]> {
    let mut bytes = [0; 32];
    for (byte, &[high, low]) in bytes.iter_mut().rev().zip(digits.as_chunks::<2>().0) {
        *byte = (nibble(high)? << 4) | nibble(low)?;
    }
    Some(bytes)
}

/// Appends a script's assembly exactly as rust-bitcoin's `Display` writes it, but each push's data
/// in one hex run: rust-bitcoin formats it a byte at a time. A malformed script is rewritten by
/// rust-bitcoin's own formatter, keeping its diagnostics.
pub(crate) fn push_asm(script: &Script, out: &mut Vec<u8>) {
    let start = out.len();
    for (n, instruction) in script.instruction_indices().enumerate() {
        let Ok((index, instruction)) = instruction else {
            out.truncate(start);
            write!(out, "{script}").expect("writing to a Vec cannot fail");
            return;
        };
        if n > 0 {
            out.push(b' ');
        }
        // The opcode as written, so a non-minimal push keeps its PUSHDATA width.
        out.extend_from_slice(OPCODE_NAMES[usize::from(script.as_bytes()[index])].as_bytes());
        if let Instruction::PushBytes(data) = instruction
            && !data.is_empty()
        {
            out.push(b' ');
            push_hex(data.as_bytes(), out);
        }
    }
}

/// Each opcode's assembly name, as rust-bitcoin's `Debug` writes it but `OP_0` for an empty push.
static OPCODE_NAMES: LazyLock<[Box<str>; 256]> = LazyLock::new(|| {
    std::array::from_fn(|byte| match Opcode::from(byte as u8) {
        OP_PUSHBYTES_0 => "OP_0".into(),
        opcode => format!("{opcode:?}").into(),
    })
});

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

/// Fixed-size byte types, each lowercase hex in text and JSON.
macro_rules! hex_bytes {
    ($($(#[$meta:meta])* $name:ident($len:literal),)+) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, ::derive_more::Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
            #[cfg_attr(feature = "storage", derive(::vecdb::Bytes))]
            pub struct $name(#[cfg_attr(feature = "schemars", schemars(with = "String"))] [u8; $len]);

            impl From<[u8; $len]> for $name {
                #[inline]
                fn from(value: [u8; $len]) -> Self {
                    Self(value)
                }
            }

            /// Panics unless `value` holds exactly the type's length.
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
    };
}

pub(crate) use hex_bytes;
