use sha2::{Digest, Sha256};

/// Bitcoin's merkle root of `hashes`; `None` of none. Double SHA-256 through `sha2`, which uses
/// the CPU's SHA instructions when present (about 6x `bitcoin_hashes` on Apple silicon, where the
/// latter has no hardware path).
pub fn merkle_root(mut hashes: Vec<[u8; 32]>) -> Option<[u8; 32]> {
    while hashes.len() > 1 {
        merkle_level(&mut hashes);
    }
    hashes.first().copied()
}

/// Replaces a merkle tree's level of two or more hashes with the level above it, in place: each
/// pair's double SHA-256, a level of odd length pairing its last hash with itself.
pub fn merkle_level(hashes: &mut Vec<[u8; 32]>) {
    if !hashes.len().is_multiple_of(2) {
        hashes.push(hashes[hashes.len() - 1]);
    }
    for i in 0..hashes.len() / 2 {
        let mut engine = Sha256::new();
        engine.update(hashes[2 * i]);
        engine.update(hashes[2 * i + 1]);
        hashes[i] = Sha256::digest(engine.finalize()).into();
    }
    hashes.truncate(hashes.len() / 2);
}
