use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock, LazyWindowStartVec, PercentPerBlock,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, SatsSigned, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{columns::Columns, part::Part};

#[derive(Traversable)]
pub struct Supply<M: StorageMode = Rw> {
    /// Total unspent supply.
    pub total: LazySpotValuePerBlock,
    /// Share of all unspent supply.
    pub share: PercentPerBlock<PartsPerMillion32, M>,
    /// Supply in profit: acquisition price is at or below the current spot price.
    pub in_profit: Part<LazySpotValuePerBlock, M>,
    /// Supply in loss: acquisition price is above the current spot price.
    pub in_loss: Part<LazySpotValuePerBlock, M>,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}
impl Supply {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
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
            share: PercentPerBlock::import(db, &id.metric_name("supply_share"), v, mappings)?,
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
    pub(crate) fn compute(
        &mut self,
        from: Height,
        c: &Columns,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        self.share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from, &c.supply, all_supply, exit,
            )?;
        self.in_profit
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_profit,
                &c.supply,
                exit,
            )?;
        self.in_loss
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_loss,
                &c.supply,
                exit,
            )?;
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [
            &mut self.share.fixed.height,
            &mut self.in_profit.share.fixed.height,
            &mut self.in_loss.share.fixed.height,
        ]
    }
}
