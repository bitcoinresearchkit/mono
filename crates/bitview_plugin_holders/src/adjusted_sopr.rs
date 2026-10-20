use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::Multiple;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec, RollingWindows, ops::Quotient,
    shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{ComputeVec, Database, Rw, StorageMode};

use crate::columns::Columns;

#[derive(Traversable)]
pub struct AdjustedSopr<M: StorageMode = Rw> {
    /// Undefined (null) when the window's adjusted value destroyed is zero.
    #[traversable(flatten)]
    pub ratio: RollingWindows<Multiple, M>,
    /// Spend-time value of the outputs it keeps (spent an hour or more after creation).
    pub value_created: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Creation-time value of the outputs it keeps (spent an hour or more after creation).
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
            value_created: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
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
            .zip(self.value_created.sum.as_array())
            .zip(self.value_destroyed.sum.as_array())
        {
            target.height.compute_binary::<_, _, Quotient<Multiple>>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        Ok(())
    }
}
