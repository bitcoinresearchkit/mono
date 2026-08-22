use bitview_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlock;

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
