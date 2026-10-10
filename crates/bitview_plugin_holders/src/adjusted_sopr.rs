use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::Ratio;
use bitview_transforms::RatioCentsOrOne;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec, RollingWindows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::columns::Columns;

#[derive(Traversable)]
pub struct AdjustedSopr<M: StorageMode = Rw> {
    pub ratio: RollingWindows<Ratio, M>,
    pub transfer_volume: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
}
impl AdjustedSopr {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            ratio: RollingWindows::import(db, &id.metric_name("adjusted_sopr"), v, mappings)?,
            transfer_volume: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &id.metric_name("adjusted_value_created"),
                v,
                &c.adjusted_volume,
                mappings,
                windows,
            ),
            value_destroyed: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &id.metric_name("adjusted_value_destroyed"),
                v,
                &c.adjusted_value_destroyed,
                mappings,
                windows,
            ),
        })
    }
    pub(crate) fn compute(&mut self, from: Height, exit: &Exit) -> Result<()> {
        for ((target, created), destroyed) in self
            .ratio
            .as_mut_array()
            .into_iter()
            .zip(self.transfer_volume.sum.as_array())
            .zip(self.value_destroyed.sum.as_array())
        {
            target.compute_binary::<_, _, RatioCentsOrOne>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.ratio
            .as_mut_array()
            .into_iter()
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
            .collect()
    }
}
