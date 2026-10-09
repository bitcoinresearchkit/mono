use bitview_primitives::{Float32, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentPerBlock};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct HashPriceValueVecs<M: StorageMode = Rw> {
    /// Per PH/s of hash rate per day, where one PH/s is 10^15 hashes per second.
    pub block: PerBlock<Float32, M>,
    /// Running all-time low of the per-PH/s series. Zero values are excluded;
    /// returns zero until the first nonzero value exists.
    pub atl: PerBlock<Float32, M>,
    /// Per-PH/s value at the represented block divided by its running all-time
    /// low, minus one. Zero marks the historical floor and positive values
    /// measure the rebound above it. Returns zero before a nonzero low exists.
    pub rebound: PercentPerBlock<PartsPerMillionSigned32, M>,
}
