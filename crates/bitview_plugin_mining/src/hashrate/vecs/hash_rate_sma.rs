use bitview_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlock;

#[derive(Traversable)]
pub struct HashRateSmaVecs<M: StorageMode = Rw> {
    pub _1w: PerBlock<StoredF64, M>,
    pub _1m: PerBlock<StoredF64, M>,
    pub _2m: PerBlock<StoredF64, M>,
    pub _1y: PerBlock<StoredF64, M>,
}
