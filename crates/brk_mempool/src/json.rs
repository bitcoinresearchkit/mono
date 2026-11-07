use std::{
    hash::{BuildHasher, DefaultHasher, RandomState},
    sync::{Arc, LazyLock},
};

use bitcoin::hashes::{Hash, sha256d};
use serde::Serialize;

/// A served JSON body and its content hash (the response's validator).
pub type JsonBody = (Arc<[u8]>, u64);

/// `value` serialized once: readers clone the `Arc`.
pub fn body(value: &impl Serialize) -> JsonBody {
    let bytes = serde_json::to_vec(value).expect("published values serialize");
    let hash = content_hash(&bytes);
    (bytes.into(), hash)
}

/// The validator of a body by its bytes: the first 8 bytes of their SHA-256d, as the server's
/// other bodies' (`bitview_query`'s `RepresentationId::content`), so no chosen content can make
/// two bodies share one.
fn content_hash(bytes: &[u8]) -> u64 {
    let digest = sha256d::Hash::hash(bytes).to_byte_array();
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}

/// Random per process: the key of every identity only this process gives (projected blocks'
/// tokens and the digests under them, revision validators), so none repeats one given before a
/// restart, and without the key no chosen content makes two share one.
static KEY: LazyLock<RandomState> = LazyLock::new(RandomState::new);

/// A hasher under [`KEY`]: SipHash, a keyed function several times faster than SHA-256 in
/// software (the ARM build has no SHA-256 instructions), for the identities hashed per
/// transaction.
pub fn keyed_hasher() -> DefaultHasher {
    KEY.build_hasher()
}

/// Mixed into a validator made from a revision counted since start.
pub fn process_nonce() -> u64 {
    KEY.hash_one(0u8)
}
