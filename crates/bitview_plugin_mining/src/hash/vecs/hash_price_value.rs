use bitview_compute::{ComputeRollingStats, ComputedVecValue};
use bitview_primitives::PartsPerMillionSigned32;
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentPerBlock, ops::RelativeChange};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use schemars::JsonSchema;
use vecdb::{ComputeVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct HashPriceValueVecs<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + JsonSchema,
{
    /// Per PH/s of hash rate per day, where one PH/s is 10^15 hashes per second.
    pub value: PerBlock<T, M>,
    /// Running all-time low of the per-PH/s series, skipping zero and null values;
    /// null until the first positive value exists.
    pub atl: PerBlock<T, M>,
    /// Per-PH/s value at the represented block divided by its running all-time
    /// low, minus one. Zero marks the historical floor and positive values
    /// measure the rebound above it. Undefined (null) before a nonzero low exists.
    pub rebound: PercentPerBlock<PartsPerMillionSigned32, M>,
}

impl<T> HashPriceValueVecs<T>
where
    T: ComputedVecValue + JsonSchema + Ord + Default + Into<f64>,
{
    pub(crate) fn compute_floor_and_rebound(
        &mut self,
        max_from: Height,
        exit: &Exit,
    ) -> Result<()> {
        self.atl
            .height
            .compute_all_time_low::<T>(max_from, &self.value.height, exit)?;
        self.rebound
            .fixed
            .height
            .compute_binary::<T, T, RelativeChange<PartsPerMillionSigned32>>(
                max_from,
                &self.value.height,
                &self.atl.height,
                exit,
            )?;
        Ok(())
    }
}
