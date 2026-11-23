use std::{fmt, str::FromStr};

use bitcoin::{Address as BitcoinAddress, ScriptBuf};
use brk_error::Error;
use derive_more::{Deref, From};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::OutputType;
use crate::{AddressBytes, sha256d};

#[cfg(feature = "storage")]
use vecdb::Formattable;

/// Bitcoin address, or the hex public key of a P2PK output.
#[derive(Debug, Clone, PartialEq, Eq, Deref, Serialize, Deserialize, From)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", schemars(
    example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
    example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    example = &"bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"
))]
pub struct Address(String);

impl TryFrom<&ScriptBuf> for Address {
    type Error = Error;
    fn try_from(script: &ScriptBuf) -> Result<Self, Self::Error> {
        Self::try_from((script, OutputType::from(script)))
    }
}

impl TryFrom<(&ScriptBuf, OutputType)> for Address {
    type Error = Error;
    fn try_from((script, output_type): (&ScriptBuf, OutputType)) -> Result<Self, Self::Error> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 => {
                // P2PK has no standard address encoding, use raw pubkey hex
                let bytes = AddressBytes::try_from((script, output_type))?;
                Ok(Self(bytes_to_hex(bytes.as_slice())))
            }
            OutputType::P2PKH | OutputType::P2SH => {
                let (pubkey, script_hash) = crate::chain().base58;
                let version = if output_type == OutputType::P2PKH {
                    pubkey
                } else {
                    script_hash
                };
                let hash = AddressBytes::script_payload(script, output_type)?;
                Ok(Self(base58check(version, hash)))
            }
            _ if output_type.is_addr() => {
                let addr = BitcoinAddress::from_script(script, crate::chain().network)
                    .map_err(|_| Error::InvalidAddress)?;
                Ok(Self(addr.to_string()))
            }
            _ => Err(Error::InvalidAddress),
        }
    }
}

impl TryFrom<&AddressBytes> for Address {
    type Error = Error;
    fn try_from(bytes: &AddressBytes) -> Result<Self, Self::Error> {
        Self::try_from(&bytes.to_script_pubkey())
    }
}

const BASE58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
/// 58^5, the largest power of 58 below 2^32: one division by it yields five digits.
const BASE58_POW5: u64 = 58 * 58 * 58 * 58 * 58;

/// Base58Check of a version byte and a 20-byte hash, as rust-bitcoin writes it. Its encoder
/// divides a byte at a time and hashes in software, which made address text most of a large
/// address history's JSON time; this divides 32-bit limbs by 58^5 and hashes with the CPU's SHA
/// instructions ([`sha256d`]).
fn base58check(version: u8, hash: &[u8]) -> String {
    let mut bytes = [0; 25];
    bytes[0] = version;
    bytes[1..21].copy_from_slice(hash);
    let checksum = sha256d(&bytes[..21]);
    bytes[21..].copy_from_slice(&checksum[..4]);
    // The 25 bytes as seven big-endian 32-bit limbs, the first holding the version byte only.
    let mut limbs = [0u32; 7];
    limbs[0] = u32::from(version);
    for (limb, chunk) in limbs[1..].iter_mut().zip(bytes[1..].as_chunks::<4>().0) {
        *limb = u32::from_be_bytes(*chunk);
    }
    // 2^200 < 58^35: seven divisions give every digit, least significant first.
    let mut digits = [0u8; 35];
    for five in digits.as_chunks_mut::<5>().0 {
        let mut rem = 0;
        for limb in &mut limbs {
            let acc = (rem << 32) | u64::from(*limb);
            *limb = (acc / BASE58_POW5) as u32;
            rem = acc % BASE58_POW5;
        }
        for digit in five {
            *digit = (rem % 58) as u8;
            rem /= 58;
        }
    }
    let zeros = bytes.iter().take_while(|&&byte| byte == 0).count();
    let len = digits.iter().rposition(|&digit| digit != 0).map_or(0, |i| i + 1);
    let mut text = String::with_capacity(zeros + len);
    text.extend(std::iter::repeat_n('1', zeros));
    text.extend(digits[..len].iter().rev().map(|&digit| char::from(BASE58[usize::from(digit)])));
    text
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = Vec::new();
    crate::bytes::push_hex(bytes, &mut hex);
    String::from_utf8(hex).expect("hex is ASCII")
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AddressBytes::parse_script(s, None)?;
        Ok(Self(s.to_string()))
    }
}

#[cfg(feature = "storage")]
impl Formattable for Address {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.0.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
