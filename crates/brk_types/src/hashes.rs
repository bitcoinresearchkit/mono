//! Bitcoin's SHA-256 based hashes with SHA-256 through `sha2`, which uses the CPU's SHA
//! instructions when present (about 6x `bitcoin_hashes` on Apple silicon, where the latter has no
//! hardware path).

use bitcoin::hashes::{Hash, ripemd160};
use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// SHA-256 of SHA-256.
pub fn sha256d(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(Sha256::digest(bytes)).into()
}

/// RIPEMD-160 of SHA-256 (only RIPEMD-160 runs in software).
pub fn hash160(bytes: &[u8]) -> [u8; 20] {
    ripemd160::Hash::hash(&Sha256::digest(bytes)).to_byte_array()
}
