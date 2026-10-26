use std::ops::Deref;

use bitview_cohort::AgeAggregateId;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock, PercentPerBlock, ops::Quotient,
};
use brk_error::Result;
use brk_types::{Sats, SatsSigned};
use vecdb::{ComputeVec, Rw, StorageMode};

use crate::{
    part::Part,
    section::{CohortCompute, Imports, Section},
};

#[derive(Traversable)]
pub struct Supply<M: StorageMode = Rw> {
    /// Total unspent supply.
    #[traversable(lead = "Supply: the bitcoin held.")]
    pub total: LazySpotValuePerBlock,
    /// Supply in profit: creation price at or below the spot price.
    #[traversable(lead = "Supply in profit: coins whose price now is at or above the one they last moved at.")]
    pub in_profit: Part<LazySpotValuePerBlock, M>,
    /// Supply in loss: creation price above the spot price.
    #[traversable(lead = "Supply in loss: coins whose price now is below the one they last moved at.")]
    pub in_loss: Part<LazySpotValuePerBlock, M>,
    /// Change in the cohort's supply over a trailing window.
    #[traversable(lead = "Supply change: how much the bitcoin held changed.")]
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}

/// The supply of a cohort but all, beside its share of all unspent supply (all's is 100%).
#[derive(Traversable)]
pub struct SharedSupply<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub supply: Supply<M>,
    /// Share of all unspent supply.
    #[traversable(lead = "Supply share: their part of all the supply.")]
    pub share: PercentPerBlock<PartsPerMillion32, M>,
}

impl Section for Supply {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        let Imports {
            db,
            v,
            mappings,
            windows,
            spot,
            ..
        } = *cx;
        let c = id.select(cx.columns);
        // All unspent outputs are the circulating supply, and their value is the market cap.
        let total = if id == AgeAggregateId::All {
            LazySpotValuePerBlock::from_sats_source_named(
                "circulating_supply",
                "market_cap",
                v,
                &c.supply,
                mappings,
                spot,
            )
        } else {
            LazySpotValuePerBlock::from_sats_source(
                &id.metric_name("supply"),
                v,
                &c.supply,
                mappings,
                spot,
            )
        };
        let part = |metric: &str, sats| {
            let name = id.metric_name(metric);
            Part::import(
                db,
                &name,
                v,
                mappings,
                LazySpotValuePerBlock::from_sats_source(&name, v, sats, mappings, spot),
            )
        };
        Ok(Self {
            total,
            in_profit: part("supply_in_profit", &c.supply_profit)?,
            in_loss: part("supply_in_loss", &c.supply_loss)?,
            delta: LazyRollingDeltasAmountFromHeight::new(
                &id.metric_name("supply_delta"),
                v,
                &c.supply,
                windows,
                mappings,
            ),
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        let CohortCompute { from, c, exit, .. } = *cx;
        self.in_profit
            .share
            .fixed
            .height
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_profit,
                &c.supply,
                exit,
            )?;
        self.in_loss
            .share
            .fixed
            .height
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_loss,
                &c.supply,
                exit,
            )?;
        Ok(())
    }
}

impl<M: StorageMode> Deref for SharedSupply<M> {
    type Target = Supply<M>;
    fn deref(&self) -> &Supply<M> {
        &self.supply
    }
}

impl Section for SharedSupply {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        Ok(Self {
            supply: Supply::import(id, cx)?,
            share: PercentPerBlock::import(
                cx.db,
                &id.metric_name("supply_share"),
                cx.v,
                cx.mappings,
            )?,
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        self.supply.compute(cx)?;
        self.share
            .fixed
            .height
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                cx.from,
                &cx.c.supply,
                cx.all_supply,
                cx.exit,
            )?;
        Ok(())
    }
}
