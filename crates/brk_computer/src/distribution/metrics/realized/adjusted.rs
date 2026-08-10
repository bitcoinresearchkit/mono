use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{BinaryTransform, Exit, ReadableVec, Rw, StorageMode};

use crate::{
    distribution::metrics::ImportConfig,
    internal::{ColumnarRollingWindows, PerBlockCumulativeRolling, RatioCents64},
};

#[derive(Traversable)]
pub struct AdjustedSopr<M: StorageMode = Rw> {
    pub ratio: ColumnarRollingWindows<StoredF64, M>,
    pub transfer_volume: PerBlockCumulativeRolling<Cents, M>,
    pub value_destroyed: PerBlockCumulativeRolling<Cents, M>,
}

impl AdjustedSopr {
    pub(crate) fn forced_import(cfg: &ImportConfig) -> Result<Self> {
        Ok(Self {
            ratio: cfg.import("asopr", Version::ONE)?,
            transfer_volume: cfg.import("adj_value_created", Version::ONE)?,
            value_destroyed: cfg.import("adj_value_destroyed", Version::ONE)?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_rest_part2(
        &mut self,
        starting_lengths: &Lengths,
        base_transfer_volume: &impl ReadableVec<Height, Cents>,
        base_value_destroyed: &impl ReadableVec<Height, Cents>,
        under_1h_transfer_volume: &impl ReadableVec<Height, Cents>,
        under_1h_value_destroyed: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.transfer_volume.cumulative.height.compute_subtract(
            starting_lengths.height,
            base_transfer_volume,
            under_1h_transfer_volume,
            exit,
        )?;
        self.value_destroyed.cumulative.height.compute_subtract(
            starting_lengths.height,
            base_value_destroyed,
            under_1h_value_destroyed,
            exit,
        )?;

        self.ratio.compute_columns2(
            starting_lengths.height,
            |window| &window.select(&self.transfer_volume.sum).height,
            |window| &window.select(&self.value_destroyed.sum).height,
            |_, transfer_volume, value_destroyed| {
                RatioCents64::apply(transfer_volume, value_destroyed)
            },
            exit,
        )?;

        Ok(())
    }
}
