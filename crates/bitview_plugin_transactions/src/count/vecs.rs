use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockFullFromCumulative;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transactions, including the coinbase transaction.
    pub total: PerBlockFullFromCumulative<Count, M>,
}
