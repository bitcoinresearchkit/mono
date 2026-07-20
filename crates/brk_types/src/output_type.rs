use bitcoin::{AddressType, ScriptBuf, opcodes::all::OP_PUSHBYTES_2};
use brk_error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;
use vecdb::{Bytes, Formattable, Pco, TransparentPco};

use crate::AddrBytes;

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
/// Type (P2PKH, P2WPKH, P2SH, P2TR, etc.)
pub enum OutputType {
    P2PK65,
    P2PK33,
    P2PKH,
    P2MS,
    P2SH,
    OpReturn,
    P2WPKH,
    P2WSH,
    P2TR,
    P2A,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
/// Output type names used by Esplora and mempool.space.
pub enum OutputTypeNormalized {
    P2PK,
    P2PKH,
    #[serde(rename = "multisig")]
    #[strum(serialize = "multisig")]
    P2MS,
    P2SH,
    #[serde(rename = "op_return")]
    #[strum(serialize = "op_return")]
    OpReturn,
    #[serde(rename = "v0_p2wpkh")]
    #[strum(serialize = "v0_p2wpkh")]
    P2WPKH,
    #[serde(rename = "v0_p2wsh")]
    #[strum(serialize = "v0_p2wsh")]
    P2WSH,
    #[serde(rename = "v1_p2tr")]
    #[strum(serialize = "v1_p2tr")]
    P2TR,
    P2A,
    Empty,
    Unknown,
}

impl OutputType {
    pub const ADDR_TYPES: [Self; 8] = [
        Self::P2PK65,
        Self::P2PK33,
        Self::P2PKH,
        Self::P2SH,
        Self::P2WPKH,
        Self::P2WSH,
        Self::P2TR,
        Self::P2A,
    ];

    fn is_valid(value: u8) -> bool {
        value <= Self::Unknown as u8
    }

    pub fn is_spendable(&self) -> bool {
        match self {
            Self::P2PK65 => true,
            Self::P2PK33 => true,
            Self::P2PKH => true,
            Self::P2MS => true,
            Self::P2SH => true,
            Self::OpReturn => false,
            Self::P2WPKH => true,
            Self::P2WSH => true,
            Self::P2TR => true,
            Self::P2A => true,
            Self::Empty => true,
            Self::Unknown => true,
        }
    }

    pub fn is_addr(&self) -> bool {
        match self {
            Self::P2PK65 => true,
            Self::P2PK33 => true,
            Self::P2PKH => true,
            Self::P2MS => false,
            Self::P2SH => true,
            Self::OpReturn => false,
            Self::P2WPKH => true,
            Self::P2WSH => true,
            Self::P2TR => true,
            Self::P2A => true,
            Self::Empty => false,
            Self::Unknown => false,
        }
    }

    pub fn is_not_addr(&self) -> bool {
        !self.is_addr()
    }

    pub fn is_unspendable(&self) -> bool {
        !self.is_spendable()
    }

    pub const fn normalized(self) -> OutputTypeNormalized {
        match self {
            Self::P2PK65 | Self::P2PK33 => OutputTypeNormalized::P2PK,
            Self::P2PKH => OutputTypeNormalized::P2PKH,
            Self::P2MS => OutputTypeNormalized::P2MS,
            Self::P2SH => OutputTypeNormalized::P2SH,
            Self::OpReturn => OutputTypeNormalized::OpReturn,
            Self::P2WPKH => OutputTypeNormalized::P2WPKH,
            Self::P2WSH => OutputTypeNormalized::P2WSH,
            Self::P2TR => OutputTypeNormalized::P2TR,
            Self::P2A => OutputTypeNormalized::P2A,
            Self::Empty => OutputTypeNormalized::Empty,
            Self::Unknown => OutputTypeNormalized::Unknown,
        }
    }

    /// Whether the address type's public key is revealed at funding time
    /// (vs. only at spending time). For P2PK33/P2PK65 the pubkey is directly
    /// in the locking script; for P2TR the tweaked output key is in the
    /// locking script. All other address types hash the pubkey/script and
    /// only reveal it on spending.
    pub fn pubkey_exposed_at_funding(&self) -> bool {
        matches!(self, Self::P2PK65 | Self::P2PK33 | Self::P2TR)
    }

    pub fn as_vec() -> Vec<Self> {
        vec![
            Self::P2PK65,
            Self::P2PK33,
            Self::P2PKH,
            Self::P2MS,
            Self::P2SH,
            Self::OpReturn,
            Self::P2WPKH,
            Self::P2WSH,
            Self::P2TR,
            Self::P2A,
            Self::Empty,
            Self::Unknown,
        ]
    }
}

impl From<&ScriptBuf> for OutputType {
    #[inline]
    fn from(script: &ScriptBuf) -> Self {
        if script.is_p2pkh() {
            Self::P2PKH
        } else if script.is_p2wpkh() {
            Self::P2WPKH
        } else if script.is_p2wsh() {
            Self::P2WSH
        } else if script.is_p2tr() {
            Self::P2TR
        } else if script.is_p2sh() {
            Self::P2SH
        } else if script.is_op_return() {
            Self::OpReturn
        } else if script.is_p2pk() {
            let bytes = script.as_bytes();

            match bytes.len() {
                67 => Self::P2PK65,
                35 => Self::P2PK33,
                _ => {
                    dbg!(bytes);
                    unreachable!()
                }
            }
        } else if script.is_multisig() {
            Self::P2MS
        } else if script.witness_version() == Some(bitcoin::WitnessVersion::V1)
            && script.len() == 4
            && script.as_bytes()[1] == OP_PUSHBYTES_2.to_u8()
            && script.as_bytes()[2..4] == [78, 115]
        {
            Self::P2A
        } else if script.is_empty() {
            Self::Empty
        } else {
            Self::Unknown
        }
    }
}

impl From<AddressType> for OutputType {
    #[inline]
    fn from(value: AddressType) -> Self {
        match value {
            AddressType::P2a => Self::P2A,
            AddressType::P2pkh => Self::P2PKH,
            AddressType::P2sh => Self::P2SH,
            AddressType::P2tr => Self::P2TR,
            AddressType::P2wpkh => Self::P2WPKH,
            AddressType::P2wsh => Self::P2WSH,
            _ => unreachable!(),
        }
    }
}

impl From<&AddrBytes> for OutputType {
    #[inline]
    fn from(bytes: &AddrBytes) -> Self {
        match bytes {
            AddrBytes::P2PK65(_) => Self::P2PK65,
            AddrBytes::P2PK33(_) => Self::P2PK33,
            AddrBytes::P2PKH(_) => Self::P2PKH,
            AddrBytes::P2SH(_) => Self::P2SH,
            AddrBytes::P2WPKH(_) => Self::P2WPKH,
            AddrBytes::P2WSH(_) => Self::P2WSH,
            AddrBytes::P2TR(_) => Self::P2TR,
            AddrBytes::P2A(_) => Self::P2A,
        }
    }
}

impl TryFrom<OutputType> for AddressType {
    type Error = Error;
    fn try_from(value: OutputType) -> Result<Self, Self::Error> {
        Ok(match value {
            OutputType::P2A => Self::P2a,
            OutputType::P2PKH => Self::P2pkh,
            OutputType::P2SH => Self::P2sh,
            OutputType::P2TR => Self::P2tr,
            OutputType::P2WPKH => Self::P2wpkh,
            OutputType::P2WSH => Self::P2wsh,
            _ => return Err(Error::UnsupportedType(format!("{:?}", value))),
        })
    }
}

impl Formattable for OutputType {
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

impl Bytes for OutputType {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(vecdb::Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        };
        let value = bytes[0];
        if !Self::is_valid(value) {
            return Err(vecdb::Error::InvalidArgument("invalid OutputType"));
        }
        // SAFETY: We validated that value is a valid variant
        let s: Self = unsafe { std::mem::transmute(value) };
        Ok(s)
    }
}

impl Pco for OutputType {
    type NumberType = u8;
}

impl TransparentPco<u8> for OutputType {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_and_normalized_names_are_distinct() {
        let cases = [
            (
                OutputType::P2PK65,
                "p2pk65",
                OutputTypeNormalized::P2PK,
                "p2pk",
            ),
            (
                OutputType::P2PK33,
                "p2pk33",
                OutputTypeNormalized::P2PK,
                "p2pk",
            ),
            (
                OutputType::P2PKH,
                "p2pkh",
                OutputTypeNormalized::P2PKH,
                "p2pkh",
            ),
            (
                OutputType::P2MS,
                "p2ms",
                OutputTypeNormalized::P2MS,
                "multisig",
            ),
            (OutputType::P2SH, "p2sh", OutputTypeNormalized::P2SH, "p2sh"),
            (
                OutputType::OpReturn,
                "opreturn",
                OutputTypeNormalized::OpReturn,
                "op_return",
            ),
            (
                OutputType::P2WPKH,
                "p2wpkh",
                OutputTypeNormalized::P2WPKH,
                "v0_p2wpkh",
            ),
            (
                OutputType::P2WSH,
                "p2wsh",
                OutputTypeNormalized::P2WSH,
                "v0_p2wsh",
            ),
            (
                OutputType::P2TR,
                "p2tr",
                OutputTypeNormalized::P2TR,
                "v1_p2tr",
            ),
            (OutputType::P2A, "p2a", OutputTypeNormalized::P2A, "p2a"),
            (
                OutputType::Empty,
                "empty",
                OutputTypeNormalized::Empty,
                "empty",
            ),
            (
                OutputType::Unknown,
                "unknown",
                OutputTypeNormalized::Unknown,
                "unknown",
            ),
        ];

        for (native, native_name, normalized, normalized_name) in cases {
            assert_eq!(native.to_string(), native_name);
            assert_eq!(native.normalized(), normalized);
            assert_eq!(
                serde_json::to_string(&native).unwrap(),
                format!(r#""{native_name}""#)
            );
            assert_eq!(
                serde_json::from_str::<OutputType>(&format!(r#""{native_name}""#)).unwrap(),
                native
            );
            assert_eq!(normalized.to_string(), normalized_name);
            assert_eq!(
                serde_json::to_string(&normalized).unwrap(),
                format!(r#""{normalized_name}""#)
            );
            assert_eq!(
                serde_json::from_str::<OutputTypeNormalized>(&format!(r#""{normalized_name}""#))
                    .unwrap(),
                normalized
            );
        }

        assert!(serde_json::from_str::<OutputType>(r#""p2pk""#).is_err());
    }
}
