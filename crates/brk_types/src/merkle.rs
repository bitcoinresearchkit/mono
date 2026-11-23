use crate::sha256d;

/// Bitcoin's merkle root of `hashes`; `None` of none. Each pair is hashed with [`sha256d`].
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
        let mut pair = [0; 64];
        pair[..32].copy_from_slice(&hashes[2 * i]);
        pair[32..].copy_from_slice(&hashes[2 * i + 1]);
        hashes[i] = sha256d(&pair);
    }
    hashes.truncate(hashes.len() / 2);
}
