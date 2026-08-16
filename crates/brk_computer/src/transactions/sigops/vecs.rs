use brk_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// BIP-141 signature-operation cost, rather than a raw signature-opcode
    /// count.
    pub total: PerBlockCumulativeRolling<StoredU64, M>,
}
