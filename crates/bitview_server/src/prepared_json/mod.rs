use axum::body::Bytes;
use brk_types::tag_hash;

use crate::{
    answer::{Answer, Content, JSON},
    etag::Etag,
};

/// Server-lifetime JSON, named once by its bytes: `c{hash:x}`.
pub struct PreparedJson {
    bytes: Bytes,
    tag: Etag,
}

impl PreparedJson {
    #[cfg(feature = "series")]
    pub fn new(value: impl serde::Serialize) -> Self {
        Self::from_bytes(Bytes::from(serde_json::to_vec(&value).unwrap()))
    }

    pub fn from_bytes(bytes: Bytes) -> Self {
        let tag = format!("c{:x}", tag_hash(&bytes)).into();
        Self { bytes, tag }
    }

    pub fn answer(&self) -> Answer {
        Answer::ready(self.tag.clone(), Content::new(JSON, self.bytes.clone())).settled(true)
    }

    #[cfg(feature = "series")]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
