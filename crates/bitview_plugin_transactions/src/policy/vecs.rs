use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::flagged::Classified;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions nonstandard under BRK's transaction-local approximation of
    /// default Bitcoin Core relay standardness at the represented block height.
    /// It checks version, size and weight, scripts and witnesses,
    /// signature-operation limits, dust, and `OP_RETURN` policy. It does not
    /// model node adoption, fee floors, mempool or package topology, conflicts or
    /// replacement, or node-specific settings. Modeled rules change after height
    /// 863,500 and at heights 905,000 and 921,000; coinbase is classified
    /// nonstandard.
    pub nonstandard: Classified<M>,
}
