use brk_types::{BlockHash, tag_hash};

/// Identity and provenance of one resolved representation.
#[derive(Debug, Clone, Copy)]
pub enum RepresentationId {
    Content(u64),
    Block(BlockHash),
}

impl RepresentationId {
    /// Identify a representation by its exact serialized bytes.
    pub fn content(bytes: &[u8]) -> Self {
        Self::Content(tag_hash(bytes))
    }
}
