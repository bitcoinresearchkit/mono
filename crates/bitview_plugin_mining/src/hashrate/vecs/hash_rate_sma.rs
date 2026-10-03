use bitview_primitives::StoredF64;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct HashRateSmaVecs<M: StorageMode = Rw> {
    /// Uses a trailing 7-day duration.
    pub _1w: PerBlock<StoredF64, M>,
    /// Uses a trailing 30-day duration.
    pub _1m: PerBlock<StoredF64, M>,
    /// Uses a trailing 60-day duration.
    pub _2m: PerBlock<StoredF64, M>,
    /// Uses a trailing 365-day duration.
    pub _1y: PerBlock<StoredF64, M>,
}
