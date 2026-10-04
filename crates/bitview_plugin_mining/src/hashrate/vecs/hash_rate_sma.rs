use bitview_primitives::Hashrate;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct HashRateSmaVecs<M: StorageMode = Rw> {
    /// Uses a trailing 7-day duration.
    pub _1w: PerBlock<Hashrate, M>,
    /// Uses a trailing 30-day duration.
    pub _1m: PerBlock<Hashrate, M>,
    /// Uses a trailing 60-day duration.
    pub _2m: PerBlock<Hashrate, M>,
    /// Uses a trailing 365-day duration.
    pub _1y: PerBlock<Hashrate, M>,
}
