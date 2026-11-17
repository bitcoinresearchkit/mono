use std::sync::Arc;

use brk_types::tag_hash;
use serde::Serialize;

/// A served JSON body and its content hash (the response's validator).
pub type JsonBody = (Arc<[u8]>, u64);

/// `value` serialized once: readers clone the `Arc`.
pub fn body(value: &impl Serialize) -> JsonBody {
    let bytes = serde_json::to_vec(value).expect("published values serialize");
    let hash = tag_hash(&bytes);
    (bytes.into(), hash)
}
