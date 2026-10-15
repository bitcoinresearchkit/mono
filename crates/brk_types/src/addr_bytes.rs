use std::str::FromStr;

use bitcoin::{Address, Network, ScriptBuf, opcodes, script::Builder};
use brk_error::Error;
use derive_more::From;

use crate::bytes::parse_hex;

use super::{
    OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes, P2TRBytes, P2WPKHBytes,
    P2WSHBytes,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub enum AddressBytes {
    P2PK65(P2PK65Bytes), // 65
    P2PK33(P2PK33Bytes), // 33
    P2PKH(P2PKHBytes),   // 20
    P2SH(P2SHBytes),     // 20
    P2WPKH(P2WPKHBytes), // 20
    P2WSH(P2WSHBytes),   // 32
    P2TR(P2TRBytes),     // 32
    P2A(P2ABytes),       // 2
}

impl AddressBytes {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            AddressBytes::P2PK65(bytes) => &bytes[..],
            AddressBytes::P2PK33(bytes) => &bytes[..],
            AddressBytes::P2PKH(bytes) => &bytes[..],
            AddressBytes::P2SH(bytes) => &bytes[..],
            AddressBytes::P2WPKH(bytes) => &bytes[..],
            AddressBytes::P2WSH(bytes) => &bytes[..],
            AddressBytes::P2TR(bytes) => &bytes[..],
            AddressBytes::P2A(bytes) => &bytes[..],
        }
    }

    /// The address payload inside `script` (what `AddressBytes` and address hashes cover).
    pub fn script_payload(script: &ScriptBuf, output_type: OutputType) -> Result<&[u8], Error> {
        let bytes = script.as_bytes();
        match output_type {
            OutputType::P2PK65 => match bytes.len() {
                67 => Ok(&bytes[1..66]),
                received => Err(Error::WrongLength {
                    expected: 67,
                    received,
                }),
            },
            OutputType::P2PK33 => match bytes.len() {
                35 => Ok(&bytes[1..34]),
                received => Err(Error::WrongLength {
                    expected: 35,
                    received,
                }),
            },
            OutputType::P2PKH => Ok(&bytes[3..23]),
            OutputType::P2SH => Ok(&bytes[2..22]),
            OutputType::P2WPKH | OutputType::P2WSH | OutputType::P2TR | OutputType::P2A => {
                Ok(&bytes[2..])
            }
            OutputType::P2MS | OutputType::Unknown | OutputType::Empty | OutputType::OpReturn => {
                Err(Error::WrongAddrType)
            }
        }
    }

    /// Reconstruct the script_pubkey from the address bytes
    pub fn to_script_pubkey(&self) -> ScriptBuf {
        match self {
            AddressBytes::P2PK65(b) => Builder::new()
                .push_slice(**b)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script(),
            AddressBytes::P2PK33(b) => Builder::new()
                .push_slice(**b)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script(),
            AddressBytes::P2PKH(b) => Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(**b)
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script(),
            AddressBytes::P2SH(b) => Builder::new()
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(**b)
                .push_opcode(opcodes::all::OP_EQUAL)
                .into_script(),
            AddressBytes::P2WPKH(b) => Builder::new().push_int(0).push_slice(**b).into_script(),
            AddressBytes::P2WSH(b) => Builder::new().push_int(0).push_slice(**b).into_script(),
            AddressBytes::P2TR(b) => Builder::new().push_int(1).push_slice(**b).into_script(),
            AddressBytes::P2A(b) => Builder::new().push_int(1).push_slice(**b).into_script(),
        }
    }
}

impl TryFrom<&ScriptBuf> for AddressBytes {
    type Error = Error;
    fn try_from(script: &ScriptBuf) -> Result<Self, Self::Error> {
        Self::try_from((script, OutputType::from(script)))
    }
}

impl TryFrom<(&ScriptBuf, OutputType)> for AddressBytes {
    type Error = Error;
    fn try_from(tuple: (&ScriptBuf, OutputType)) -> Result<Self, Self::Error> {
        let (script, output_type) = tuple;
        let bytes = Self::script_payload(script, output_type)?;

        Ok(match output_type {
            OutputType::P2PK65 => Self::P2PK65(P2PK65Bytes::from(bytes)),
            OutputType::P2PK33 => Self::P2PK33(P2PK33Bytes::from(bytes)),
            OutputType::P2PKH => Self::P2PKH(P2PKHBytes::from(bytes)),
            OutputType::P2SH => Self::P2SH(P2SHBytes::from(bytes)),
            OutputType::P2WPKH => Self::P2WPKH(P2WPKHBytes::from(bytes)),
            OutputType::P2WSH => Self::P2WSH(P2WSHBytes::from(bytes)),
            OutputType::P2TR => Self::P2TR(P2TRBytes::from(bytes)),
            OutputType::P2A => Self::P2A(P2ABytes::from(bytes)),
            OutputType::P2MS | OutputType::Unknown | OutputType::Empty | OutputType::OpReturn => {
                unreachable!()
            }
        })
    }
}

impl AddressBytes {
    /// Parse an address string to a ScriptBuf. A P2PK "address" is its 33- or 65-byte key field
    /// in hex, as the index keeps it: P2PK-shaped, not necessarily a valid public key.
    pub fn addr_to_script(addr: &str) -> Result<ScriptBuf, Error> {
        if let Ok(addr) = Address::from_str(addr) {
            if !addr.is_valid_for_network(Network::Bitcoin) {
                return Err(Error::InvalidNetwork);
            }
            let addr = addr.assume_checked();
            Ok(addr.script_pubkey())
        } else {
            p2pk_script(addr).ok_or(Error::InvalidAddress)
        }
    }
}

impl FromStr for AddressBytes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let script = Self::addr_to_script(s)?;
        let output_type = OutputType::from(&script);
        Self::try_from((&script, output_type))
    }
}

/// `PUSH<len> <key field> OP_CHECKSIG` for a 33- or 65-byte key field in hex.
fn p2pk_script(hex: &str) -> Option<ScriptBuf> {
    let key = match hex.len() {
        66 => parse_hex::<33>(hex)?.to_vec(),
        130 => parse_hex::<65>(hex)?.to_vec(),
        _ => return None,
    };
    // Push opcodes 1-75 are their byte counts.
    let mut script = Vec::with_capacity(key.len() + 2);
    script.push(key.len() as u8);
    script.extend_from_slice(&key);
    script.push(opcodes::all::OP_CHECKSIG.to_u8());
    Some(ScriptBuf::from_bytes(script))
}
