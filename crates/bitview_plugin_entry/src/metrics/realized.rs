use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::Ratio;
use bitview_transforms::RatioCentsOrOne;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, LazyValuePerBlockCumulativeRolling, LazyWindowStartVec,
    RollingWindows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{Database, Rw, StorageMode};

use super::Sources;

#[derive(Traversable)]
pub struct RealizedMetrics<M: StorageMode = Rw> {
    /// Profit realized by outputs spent from this cohort.
    profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Loss realized by outputs spent from this cohort.
    loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Realized profit minus realized loss.
    net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    /// Creation-time value of the outputs spent.
    value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Spent output profit ratio (SOPR): spend-time value of the outputs spent over the
    /// window divided by their creation-time value.
    sopr: RollingWindows<Ratio, M>,
}

impl RealizedMetrics {
    pub(crate) fn import(
        db: &Database,
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let flow = |metric, source| {
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name(metric),
                version,
                source,
                mappings,
                windows,
            )
        };
        Ok(Self {
            profit: flow(
                "realized_profit",
                sources.realized_profit.cumulative_source(),
            ),
            loss: flow("realized_loss", sources.realized_loss.cumulative_source()),
            net_pnl: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name("net_realized_pnl"),
                version,
                sources.realized_net_pnl.cumulative_source(),
                mappings,
                windows,
            ),
            value_destroyed: flow(
                "value_destroyed",
                sources.value_destroyed.cumulative_source(),
            ),
            sopr: RollingWindows::import(db, &name("sopr"), version, mappings)?,
        })
    }

    pub(crate) fn compute(
        &mut self,
        from: Height,
        transfer_volume: &LazyValuePerBlockCumulativeRolling,
        exit: &Exit,
    ) -> Result<()> {
        for ((target, created), destroyed) in self
            .sopr
            .as_mut_array()
            .into_iter()
            .zip(transfer_volume.sum.0.as_array())
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
}
