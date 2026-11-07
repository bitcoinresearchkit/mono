use std::{
    hash::{BuildHasher, RandomState},
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

/// The validator of a body by its bytes: the first 8 bytes of their SHA-256d, as every content
/// identity the server gives (`bitview_query`'s `RepresentationId::content`), so no chosen
/// content can make two bodies share one.
fn content_hash(bytes: &[u8]) -> u64 {
    let digest = sha256d::Hash::hash(bytes).to_byte_array();
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}

/// Random per process: validators made from a revision counted since start never repeat one
/// given before a restart.
pub fn process_nonce() -> u64 {
    static NONCE: LazyLock<u64> = LazyLock::new(|| RandomState::new().hash_one(0u8));
    *NONCE
}
