use std::str::FromStr;

use brk_types::{AddressBytes, OutputType};

use crate::{BitviewError, Result};

/// BRK address type and raw payload bytes used by the hash-prefix index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressPayload {
    pub address_type: OutputType,
    pub payload: Vec<u8>,
}

/// BRK address type and leading hex nibbles of the address-payload hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressHashPrefix {
    pub address_type: OutputType,
    pub prefix: String,
}

/// Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`.
pub fn address_payload_hash_prefix(payload: &[u8], nibbles: usize) -> Result<String> {
    if payload.is_empty() {
        return Err(BitviewError::new("Expected a non-empty address payload"));
    }
    if payload.len() > 65 {
        return Err(BitviewError::new(
            "Expected at most 65 address payload bytes",
        ));
    }
    if !(1..=16).contains(&nibbles) {
        return Err(BitviewError::new(
            "Expected hash-prefix length from 1 to 16 hex nibbles",
        ));
    }
    let mut prefix = format!("{:016x}", rapidhash::v3::rapidhash_v3(payload));
    prefix.truncate(nibbles);
    Ok(prefix)
}

pub(crate) fn validate_address_payload_for_type(
    address_type: OutputType,
    payload: &[u8],
) -> Result<()> {
    let expected: &[usize] = match address_type {
        OutputType::P2A => &[2],
        OutputType::P2PK33 => &[33],
        OutputType::P2PK65 => &[65],
        OutputType::P2PKH | OutputType::P2SH | OutputType::P2WPKH => &[20],
        OutputType::P2WSH | OutputType::P2TR => &[32],
        OutputType::P2MS | OutputType::OpReturn | OutputType::Empty | OutputType::Unknown => {
            return Err(BitviewError::new(format!(
                "Unsupported address type for address payload hash-prefix: {address_type:?}"
            )));
        }
    };

    if !expected.contains(&payload.len()) {
        let joined = expected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" or ");
        return Err(BitviewError::new(format!(
            "Expected {address_type} address payload length {joined} bytes"
        )));
    }

    Ok(())
}

fn decode_address(address: &str) -> Result<AddressBytes> {
    if address.is_empty() {
        return Err(BitviewError::new("Expected an address string"));
    }
    AddressBytes::from_str(address).map_err(|e| BitviewError::new(e.to_string()))
}

/// Decode a mainnet Bitcoin address into the BRK address type and raw payload bytes.
pub fn decode_address_payload(address: &str) -> Result<AddressPayload> {
    let bytes = decode_address(address)?;
    Ok(AddressPayload {
        address_type: OutputType::from(&bytes),
        payload: bytes.as_slice().to_vec(),
    })
}

/// Decode a mainnet Bitcoin address and compute its hash prefix.
pub fn address_hash_prefix(address: &str, nibbles: usize) -> Result<AddressHashPrefix> {
    let bytes = decode_address(address)?;
    Ok(AddressHashPrefix {
        address_type: OutputType::from(&bytes),
        prefix: address_payload_hash_prefix(bytes.as_slice(), nibbles)?,
    })
}
