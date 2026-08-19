use bitview_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlockFullFromCumulative;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Number of transactions, including the coinbase transaction.
    pub total: PerBlockFullFromCumulative<StoredU64, M>,
}
