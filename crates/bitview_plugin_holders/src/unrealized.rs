use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned32};
use bitview_transforms::{Quotient, RatioDollars};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyRatioPerBlock, RatioPerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{columns::Columns, supply::Supply};

#[derive(Traversable)]
pub struct Unrealized<M: StorageMode = Rw> {
    pub profit: LazyFiatPerBlock<Cents>,
    pub loss: LazyFiatPerBlock<Cents>,
    pub net_pnl: LazyFiatPerBlock<CentsSigned>,
    pub gross_pnl: LazyFiatPerBlock<Cents>,
    /// Net unrealized profit/loss (NUPL): net unrealized profit and loss divided by the
    /// cohort's market cap.
    pub nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
    /// Unrealized profit divided by the market cap of all unspent supply.
    pub profit_to_market_cap: RatioPerBlock<PartsPerMillion32, M>,
    /// Unrealized loss divided by the market cap of all unspent supply.
    pub loss_to_market_cap: RatioPerBlock<PartsPerMillion32, M>,
    /// Unrealized profit divided by the cohort's market cap.
    pub profit_to_own_market_cap: RatioPerBlock<PartsPerMillion32, M>,
    /// Unrealized loss divided by the cohort's market cap.
    pub loss_to_own_market_cap: RatioPerBlock<PartsPerMillion32, M>,
    /// Net unrealized profit and loss divided by the gross.
    pub net_pnl_to_gross_pnl: RatioPerBlock<PartsPerMillionSigned32, M>,
}
impl Unrealized {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
    ) -> Result<Self> {
        let fiat = |metric: &str, source| {
            LazyFiatPerBlock::from_cents_source(&id.metric_name(metric), v, source, mappings)
        };
        let ratio = |metric: &str| RatioPerBlock::import(db, &id.metric_name(metric), v, mappings);
        Ok(Self {
            profit: fiat("unrealized_profit", &c.unrealized_profit),
            loss: fiat("unrealized_loss", &c.unrealized_loss),
            net_pnl: LazyFiatPerBlock::from_cents_source(
                &id.metric_name("net_unrealized_pnl"),
                v,
                &c.unrealized_net_pnl,
                mappings,
            ),
            gross_pnl: fiat("gross_unrealized_pnl", &c.unrealized_gross_pnl),
            nupl: LazyRatioPerBlock::from_height_source(
                &id.metric_name("nupl"),
                v,
                &c.nupl,
                mappings,
            ),
            profit_to_market_cap: ratio("unrealized_profit_to_market_cap")?,
            loss_to_market_cap: ratio("unrealized_loss_to_market_cap")?,
            profit_to_own_market_cap: ratio("unrealized_profit_to_own_market_cap")?,
            loss_to_own_market_cap: ratio("unrealized_loss_to_own_market_cap")?,
            net_pnl_to_gross_pnl: RatioPerBlock::import(
                db,
                &id.metric_name("net_unrealized_pnl_to_gross_pnl"),
                v,
                mappings,
            )?,
        })
    }
    pub(crate) fn compute(
        &mut self,
        from: Height,
        c: &Columns,
        supply: &Supply,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.profit_to_market_cap
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_profit,
                all_market_cap,
                exit,
            )?;
        self.loss_to_market_cap
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_loss,
                all_market_cap,
                exit,
            )?;
        self.profit_to_own_market_cap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &self.profit.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        self.loss_to_own_market_cap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &self.loss.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        self.net_pnl_to_gross_pnl
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned32>>(
                from,
                &c.unrealized_net_pnl,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            &mut self.profit_to_market_cap.fixed.height,
            &mut self.loss_to_market_cap.fixed.height,
            &mut self.profit_to_own_market_cap.fixed.height,
            &mut self.loss_to_own_market_cap.fixed.height,
            &mut self.net_pnl_to_gross_pnl.fixed.height,
        ]
    }
}
