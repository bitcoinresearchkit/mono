use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, Ratio};
use bitview_transforms::{Quotient, RatioCentsOrOne};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec, RatioRollingWindows, RollingWindows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{activity::Activity, adjusted_sopr::AdjustedSopr, columns::Columns};

#[derive(Traversable)]
pub struct Realized<M: StorageMode = Rw> {
    pub profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    pub gross_pnl: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Realized profit divided by realized loss over the window.
    pub profit_to_loss: RollingWindows<Ratio, M>,
    /// Creation-time value of the outputs spent.
    pub value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub peak_regret: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Spent output profit ratio (SOPR): spend-time value of the outputs spent over the
    /// window divided by their creation-time value.
    pub sopr: RollingWindows<Ratio, M>,
    /// SOPR without the outputs spent within an hour of their creation.
    pub adjusted_sopr: AdjustedSopr<M>,
    /// Gross realized profit and loss over the window divided by the realized cap.
    pub sell_side_risk_ratio: RatioRollingWindows<PartsPerMillion32, M>,
}
impl Realized {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let flow = |metric: &str, source| {
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &id.metric_name(metric),
                v,
                source,
                mappings,
                windows,
            )
        };
        Ok(Self {
            profit: flow("realized_profit", &c.profit),
            loss: flow("realized_loss", &c.loss),
            net_pnl: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &id.metric_name("net_realized_pnl"),
                v,
                &c.net_pnl,
                mappings,
                windows,
            ),
            gross_pnl: flow("gross_realized_pnl", &c.gross_pnl),
            profit_to_loss: RollingWindows::import(
                db,
                &id.metric_name("realized_profit_to_loss"),
                v,
                mappings,
            )?,
            value_destroyed: flow("value_destroyed", &c.value_destroyed),
            peak_regret: flow("realized_peak_regret", &c.peak_regret),
            sopr: RollingWindows::import(db, &id.metric_name("sopr"), v, mappings)?,
            adjusted_sopr: AdjustedSopr::import(db, id, v, c, mappings, windows)?,
            sell_side_risk_ratio: RatioRollingWindows::import(
                db,
                &id.metric_name("sell_side_risk_ratio"),
                v,
                mappings,
            )?,
        })
    }
    pub(crate) fn compute(
        &mut self,
        from: Height,
        c: &Columns,
        activity: &Activity,
        exit: &Exit,
    ) -> Result<()> {
        for ((target, profit), loss) in self
            .profit_to_loss
            .as_mut_array()
            .into_iter()
            .zip(self.profit.sum.as_array())
            .zip(self.loss.sum.as_array())
        {
            target.compute_binary::<_, _, RatioCentsOrOne>(
                from,
                &profit.cents.height,
                &loss.cents.height,
                exit,
            )?;
        }
        for ((target, created), destroyed) in self
            .sopr
            .as_mut_array()
            .into_iter()
            .zip(activity.transfer_volume.sum.0.as_array())
            .zip(self.value_destroyed.sum.as_array())
        {
            target.compute_binary::<_, _, RatioCentsOrOne>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        self.adjusted_sopr.compute(from, exit)?;
        for (target, pnl) in self
            .sell_side_risk_ratio
            .as_mut_array()
            .into_iter()
            .zip(self.gross_pnl.sum.as_array())
        {
            target
                .fixed
                .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                    from,
                    &pnl.cents.height,
                    &c.cap,
                    exit,
                )?;
        }
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            profit_to_loss,
            sopr,
            adjusted_sopr,
            sell_side_risk_ratio,
            ..
        } = self;
        profit_to_loss
            .as_mut_array()
            .into_iter()
            .chain(sopr.as_mut_array())
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
            .chain(adjusted_sopr.stored_vecs_mut())
            .chain(
                sell_side_risk_ratio
                    .as_mut_array()
                    .into_iter()
                    .map(|v| &mut v.fixed.height as &mut dyn AnyStoredVec),
            )
    }
}
