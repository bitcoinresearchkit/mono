use bitview_cohort::AgeAggregateId;
use bitview_primitives::{Multiple, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, RatioRollingWindows, RollingWindows, ops::Quotient,
};
use brk_error::Result;
use brk_types::{Cents, CentsSigned};
use vecdb::{ComputeVec, Rw, StorageMode};

use crate::section::{CohortCompute, Imports, Section};

#[derive(Traversable)]
pub struct Realized<M: StorageMode = Rw> {
    /// Profit realized by the cohort's outputs: spending value minus
    /// creation-date value, counted only for profitable spends.
    #[traversable(lead = "Realized profit: what sellers gained over what their coins cost.")]
    pub profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Loss realized by the cohort's outputs: creation-date value minus
    /// spending value, counted only for losing spends.
    #[traversable(lead = "Realized loss: what sellers lost against what their coins cost.")]
    pub loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Net realized profit and loss of the cohort's outputs when spent:
    /// realized profit minus realized loss.
    #[traversable(lead = "Net realized profit and loss: realized profit minus realized loss.")]
    pub net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    /// Gross realized profit and loss: realized profit plus realized loss.
    #[traversable(lead = "Gross realized profit and loss: realized profit plus realized loss.")]
    pub gross_pnl: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Realized profit divided by realized loss over the window; undefined (null) when that
    /// realized loss is zero.
    #[traversable(
        lead = "Realized profit to loss: dollars of profit taken for each dollar of loss."
    )]
    pub profit_to_loss: RollingWindows<Multiple, M>,
    /// Creation-time value of the outputs spent.
    #[traversable(lead = "Value destroyed: what the coins spent had cost their holders.")]
    pub value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Peak regret: the value sellers forgo relative to each spent output's
    /// highest spot price from its creation block through its spending block.
    #[traversable(
        lead = "Peak regret: what sellers gave up against the highest price their coins saw while they held them."
    )]
    pub peak_regret: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Spent output profit ratio (SOPR): spend-time value in US dollars of the outputs spent
    /// over the window divided by their creation-time value (value destroyed); undefined (null)
    /// when that value is zero.
    #[traversable(
        lead = "SOPR (spent output profit ratio): what coins sold for against what they cost. Above 1, sellers take profits; below 1, they sell at a loss."
    )]
    pub sopr: RollingWindows<Multiple, M>,
    /// Gross realized profit and loss over the window divided by the realized cap; undefined
    /// (null) when the realized cap is zero or the window reaches before the first exchange price
    /// (block 74,641).
    #[traversable(
        lead = "Sell-side risk: realized profit and loss together, against the realized cap: how much value sellers move."
    )]
    pub sell_side_risk_ratio: RatioRollingWindows<PartsPerMillion32, M>,
}
impl Section for Realized {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        let Imports {
            db,
            v,
            mappings,
            windows,
            ..
        } = *cx;
        let c = id.select(cx.columns);
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
            sell_side_risk_ratio: RatioRollingWindows::import(
                db,
                &id.metric_name("sell_side_risk_ratio"),
                v,
                mappings,
            )?,
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        let CohortCompute {
            from,
            c,
            activity,
            exit,
            ..
        } = *cx;
        for ((target, profit), loss) in self
            .profit_to_loss
            .as_mut_array()
            .into_iter()
            .zip(self.profit.sum.as_array())
            .zip(self.loss.sum.as_array())
        {
            target.height.compute_binary::<_, _, Quotient<Multiple>>(
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
            target.height.compute_binary::<_, _, Quotient<Multiple>>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        for (target, pnl) in self
            .sell_side_risk_ratio
            .as_mut_array()
            .into_iter()
            .zip(self.gross_pnl.sum.as_array())
        {
            target
                .fixed
                .height
                .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                    from,
                    &pnl.cents.height,
                    &c.cap,
                    exit,
                )?;
        }
        Ok(())
    }
}
