use bitview_compute::ComputedVecValue;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentPerBlock};
use schemars::JsonSchema;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct HashPriceValueVecs<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + JsonSchema,
{
    /// Per PH/s of hash rate per day, where one PH/s is 10^15 hashes per second.
    pub block: PerBlock<T, M>,
    /// Running all-time low of the per-PH/s series, skipping zero and null values;
    /// null until the first positive value exists.
    pub atl: PerBlock<T, M>,
    /// Per-PH/s value at the represented block divided by its running all-time
    /// low, minus one. Zero marks the historical floor and positive values
    /// measure the rebound above it. Undefined (null) before a nonzero low exists.
    pub rebound: PercentPerBlock<PartsPerMillionSigned32, M>,
}
