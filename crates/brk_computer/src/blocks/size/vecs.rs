use brk_traversable::Traversable;
use brk_types::{StoredU64, Weight};
use vecdb::{Rw, StorageMode};

use crate::internal::{CachedPerBlockRolling, PerBlockFull};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Virtual size of the block in vbytes, computed as the block weight in
    /// weight units divided by four and rounded down.
    pub vbytes: PerBlockFull<StoredU64, Weight, M>,
    /// Total serialized block size in bytes, including the header,
    /// transaction-count CompactSize, and witness data.
    pub size: CachedPerBlockRolling<StoredU64, M>,
}
