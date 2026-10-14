use bitview_compute::{AverageWindow, Retained};
use bitview_primitives::Hashrate;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct HashRateSmaVecs<M: StorageMode = Rw> {
    /// Uses a trailing 7-day duration.
    pub _1w: PerBlock<Hashrate, M>,
    /// Uses a trailing 30-day duration. With the 60-day average, it forms the Hash Ribbons.
    pub _1m: PerBlock<Hashrate, M>,
    /// Uses a trailing 60-day duration. With the 30-day average, it forms the Hash Ribbons.
    pub _2m: PerBlock<Hashrate, M>,
    /// Uses a trailing 365-day duration.
    pub _1y: PerBlock<Hashrate, M>,
    /// The kept 1w, 1m, 2m and 1y average windows.
    pub(crate) windows: M::WriteOnly<[Retained<AverageWindow>; 4]>,
}
