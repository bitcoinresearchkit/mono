use std::{borrow::Borrow, ops::Deref};

use bitview_cohort::AgeAggregateId;
use bitview_primitives::PartsPerMillion32;
use bitview_distribution::metrics::holding;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyIndexedVec, LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock, PercentPerBlock,
    ops::Quotient,
};
use brk_error::Result;
use brk_types::{Height, Sats};
use vecdb::{ComputeVec, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    part::import_part,
    section::{CohortCompute, Imports, Section},
};

/// A holder group's supply, its parts' shares stored.
pub type Supply<M = Rw> = holding::Supply<PercentPerBlock<PartsPerMillion32, M>>;

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
        // Profit and loss split the cohort's coins: the loss side is what profit leaves.
        let supply_loss = LazyIndexedVec::new(
            &id.metric_name("supply_in_loss_sats_source"),
            v,
            &c.supply,
            &c.supply_profit,
            |_, supply: Sats, in_profit: Sats| supply - in_profit,
        );
        Ok(Self {
            total,
            in_profit: spot_part(cx, &id.metric_name("supply_in_profit"), &c.supply_profit)?,
            in_loss: spot_part(cx, &id.metric_name("supply_in_loss"), &supply_loss)?,
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
                &self.in_loss.value.sats.height,
                &c.supply,
                exit,
            )?;
        Ok(())
    }
}

fn spot_part(
    cx: &Imports,
    name: &str,
    sats: &impl ReadableCloneableVec<Height, Sats>,
) -> Result<holding::Part<LazySpotValuePerBlock, PercentPerBlock<PartsPerMillion32>>> {
    import_part(
        cx.db,
        name,
        cx.v,
        cx.mappings,
        LazySpotValuePerBlock::from_sats_source(name, cx.v, sats, cx.mappings, cx.spot),
    )
}

impl<M: StorageMode> Deref for SharedSupply<M> {
    type Target = Supply<M>;
    fn deref(&self) -> &Supply<M> {
        &self.supply
    }
}

impl<M: StorageMode> Borrow<Supply<M>> for SharedSupply<M> {
    fn borrow(&self) -> &Supply<M> {
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
