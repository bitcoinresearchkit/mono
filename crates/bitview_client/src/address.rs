use std::str::FromStr;

use brk_types::{AddressBytes, OutputType};

use crate::{BitviewError, Result};

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
    let Some(len) = address_type.payload_len() else {
        return Err(BitviewError::new(format!(
            "Unsupported address type for address payload hash-prefix: {address_type:?}"
        )));
    };
    if payload.len() != len {
        return Err(BitviewError::new(format!(
            "Expected {address_type} address payload length {len} bytes"
        )));
    }
    Ok(())
}

/// Decode a mainnet Bitcoin address into its typed payload: `OutputType::from(&bytes)` is its
/// address type, `bytes.as_slice()` the raw payload the hash-prefix index hashes.
pub fn decode_address_payload(address: &str) -> Result<AddressBytes> {
    if address.is_empty() {
        return Err(BitviewError::new("Expected an address string"));
    }
    AddressBytes::from_str(address).map_err(|e| BitviewError::new(e.to_string()))
}

/// Decode a mainnet Bitcoin address and compute its hash prefix.
pub fn address_hash_prefix(address: &str, nibbles: usize) -> Result<AddressHashPrefix> {
    let bytes = decode_address_payload(address)?;
    Ok(AddressHashPrefix {
        address_type: OutputType::from(&bytes),
        prefix: address_payload_hash_prefix(bytes.as_slice(), nibbles)?,
    })
}
