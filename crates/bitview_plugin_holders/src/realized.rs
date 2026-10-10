use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64, Ratio};
use bitview_transforms::{Quotient, RatioCentsOrOne};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyFiatPerBlockCumulativeWithSums, LazyFiatPerBlockWithDeltas,
    LazyWindowStartVec, RatioPerBlock, RatioRollingWindows, RollingWindows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{activity::Activity, adjusted_sopr::AdjustedSopr, columns::Columns, part::Part};

#[derive(Traversable)]
pub struct Realized<M: StorageMode = Rw> {
    pub cap: LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Realized cap of the supply in profit.
    #[traversable(wrap = "cap", rename = "in_profit")]
    pub cap_in_profit: Part<LazyFiatPerBlock<Cents>, M>,
    /// Realized cap of the supply in loss.
    #[traversable(wrap = "cap", rename = "in_loss")]
    pub cap_in_loss: Part<LazyFiatPerBlock<Cents>, M>,
    pub profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    /// Net realized profit and loss over the trailing 30 days divided by the market cap of
    /// all unspent supply.
    pub net_pnl_1m_to_market_cap: RatioPerBlock<PartsPerMillionSigned64, M>,
    /// Net realized profit and loss over the trailing 30 days divided by the cohort's realized
    /// cap.
    pub net_pnl_1m_to_realized_cap: RatioPerBlock<PartsPerMillionSigned64, M>,
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
        let cap_part = |metric: &str, source| {
            let name = id.metric_name(metric);
            Part::import(
                db,
                &name,
                v,
                mappings,
                LazyFiatPerBlock::from_cents_source(&name, v, source, mappings),
            )
        };
        Ok(Self {
            cap: LazyFiatPerBlockWithDeltas::from_cents_source(
                &id.metric_name("realized_cap"),
                v,
                &c.cap,
                v,
                mappings,
                windows,
            ),
            cap_in_profit: cap_part("realized_cap_in_profit", &c.cap_profit)?,
            cap_in_loss: cap_part("realized_cap_in_loss", &c.cap_loss)?,
            profit: flow("realized_profit", &c.profit),
            loss: flow("realized_loss", &c.loss),
            net_pnl: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &id.metric_name("net_realized_pnl"),
                v,
                &c.net_pnl,
                mappings,
                windows,
            ),
            net_pnl_1m_to_market_cap: RatioPerBlock::import(
                db,
                &id.metric_name("net_realized_pnl_1m_to_market_cap"),
                v,
                mappings,
            )?,
            net_pnl_1m_to_realized_cap: RatioPerBlock::import(
                db,
                &id.metric_name("net_realized_pnl_1m_to_realized_cap"),
                v,
                mappings,
            )?,
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
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.cap_in_profit
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.cap_profit,
                &c.cap,
                exit,
            )?;
        self.cap_in_loss
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(from, &c.cap_loss, &c.cap, exit)?;
        let net_pnl_1m = &self.net_pnl.sum._1m.cents.height;
        self.net_pnl_1m_to_market_cap
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned64>>(
                from,
                net_pnl_1m,
                all_market_cap,
                exit,
            )?;
        self.net_pnl_1m_to_realized_cap
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned64>>(
                from, net_pnl_1m, &c.cap, exit,
            )?;
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
            cap_in_profit,
            cap_in_loss,
            net_pnl_1m_to_market_cap,
            net_pnl_1m_to_realized_cap,
            profit_to_loss,
            sopr,
            adjusted_sopr,
            sell_side_risk_ratio,
            ..
        } = self;
        [
            &mut cap_in_profit.share.fixed.height as &mut dyn AnyStoredVec,
            &mut cap_in_loss.share.fixed.height,
            &mut net_pnl_1m_to_market_cap.fixed.height,
            &mut net_pnl_1m_to_realized_cap.fixed.height,
        ]
        .into_iter()
        .chain(
            profit_to_loss
                .as_mut_array()
                .into_iter()
                .chain(sopr.as_mut_array())
                .map(|v| &mut v.height as &mut dyn AnyStoredVec),
        )
        .chain(adjusted_sopr.stored_vecs_mut())
        .chain(
            sell_side_risk_ratio
                .as_mut_array()
                .into_iter()
                .map(|v| &mut v.fixed.height as &mut dyn AnyStoredVec),
        )
    }
}
