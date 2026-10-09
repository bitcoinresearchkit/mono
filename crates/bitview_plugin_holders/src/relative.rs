use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned32, PartsPerMillionSigned64};
use bitview_transforms::{Quotient, RatioDollars};
use bitview_traversable::Traversable;
use bitview_vecs::FixedRatioPerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::metrics::Metrics;
#[derive(Traversable)]
pub struct Relative<M: StorageMode = Rw> {
    pub supply_dominance: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub supply_in_profit_share: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub supply_in_loss_share: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_mcap: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_mcap: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_own_mcap: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_own_mcap: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_own_gross_pnl: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_own_gross_pnl: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub net_unrealized_pnl_to_own_gross_pnl: FixedRatioPerBlock<PartsPerMillionSigned32, M>,
    pub invested_capital_in_profit_share: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub invested_capital_in_loss_share: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub realized_cap_to_own_mcap: FixedRatioPerBlock<PartsPerMillion32, M>,
    pub net_pnl_change_1m_to_mcap: FixedRatioPerBlock<PartsPerMillionSigned64, M>,
    pub net_pnl_change_1m_to_rcap: FixedRatioPerBlock<PartsPerMillionSigned64, M>,
}
impl Relative {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        mappings: &Mappings,
    ) -> Result<Self> {
        Ok(Self {
            supply_dominance: FixedRatioPerBlock::import(
                db,
                &id.metric_name("supply_dominance"),
                v,
                mappings,
            )?,
            supply_in_profit_share: FixedRatioPerBlock::import(
                db,
                &id.metric_name("supply_in_profit_share"),
                v,
                mappings,
            )?,
            supply_in_loss_share: FixedRatioPerBlock::import(
                db,
                &id.metric_name("supply_in_loss_share"),
                v,
                mappings,
            )?,
            unrealized_profit_to_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_profit_to_mcap"),
                v,
                mappings,
            )?,
            unrealized_loss_to_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_loss_to_mcap"),
                v,
                mappings,
            )?,
            unrealized_profit_to_own_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_profit_to_own_mcap"),
                v,
                mappings,
            )?,
            unrealized_loss_to_own_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_loss_to_own_mcap"),
                v,
                mappings,
            )?,
            unrealized_profit_to_own_gross_pnl: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_profit_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            unrealized_loss_to_own_gross_pnl: FixedRatioPerBlock::import(
                db,
                &id.metric_name("unrealized_loss_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            net_unrealized_pnl_to_own_gross_pnl: FixedRatioPerBlock::import(
                db,
                &id.metric_name("net_unrealized_pnl_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            invested_capital_in_profit_share: FixedRatioPerBlock::import(
                db,
                &id.metric_name("invested_capital_in_profit_share"),
                v,
                mappings,
            )?,
            invested_capital_in_loss_share: FixedRatioPerBlock::import(
                db,
                &id.metric_name("invested_capital_in_loss_share"),
                v,
                mappings,
            )?,
            realized_cap_to_own_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("realized_cap_to_own_mcap"),
                v,
                mappings,
            )?,
            net_pnl_change_1m_to_mcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("net_pnl_change_1m_to_mcap"),
                v,
                mappings,
            )?,
            net_pnl_change_1m_to_rcap: FixedRatioPerBlock::import(
                db,
                &id.metric_name("net_pnl_change_1m_to_rcap"),
                v,
                mappings,
            )?,
        })
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.supply_dominance.ppm.height,
            &mut self.supply_in_profit_share.ppm.height,
            &mut self.supply_in_loss_share.ppm.height,
            &mut self.unrealized_profit_to_mcap.ppm.height,
            &mut self.unrealized_loss_to_mcap.ppm.height,
            &mut self.unrealized_profit_to_own_mcap.ppm.height,
            &mut self.unrealized_loss_to_own_mcap.ppm.height,
            &mut self.unrealized_profit_to_own_gross_pnl.ppm.height,
            &mut self.unrealized_loss_to_own_gross_pnl.ppm.height,
            &mut self.net_unrealized_pnl_to_own_gross_pnl.ppm.height,
            &mut self.invested_capital_in_profit_share.ppm.height,
            &mut self.invested_capital_in_loss_share.ppm.height,
            &mut self.realized_cap_to_own_mcap.ppm.height,
            &mut self.net_pnl_change_1m_to_mcap.ppm.height,
            &mut self.net_pnl_change_1m_to_rcap.ppm.height,
        ]
    }
}

impl Metrics {
    pub(crate) fn compute_relative(
        &mut self,
        from: Height,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            relative,
            columns: c,
            supply,
            realized,
            unrealized,
            ..
        } = self;
        relative
            .supply_dominance
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from, &c.supply, all_supply, exit,
            )?;
        relative
            .supply_in_profit_share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_profit,
                &c.supply,
                exit,
            )?;
        relative
            .supply_in_loss_share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.supply_loss,
                &c.supply,
                exit,
            )?;
        relative
            .unrealized_profit_to_mcap
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_profit,
                all_market_cap,
                exit,
            )?;
        relative
            .unrealized_loss_to_mcap
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_loss,
                all_market_cap,
                exit,
            )?;
        relative
            .unrealized_profit_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &unrealized.profit.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        relative
            .unrealized_loss_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &unrealized.loss.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        relative
            .unrealized_profit_to_own_gross_pnl
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_profit,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        relative
            .unrealized_loss_to_own_gross_pnl
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.unrealized_loss,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        relative
            .net_unrealized_pnl_to_own_gross_pnl
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned32>>(
                from,
                &c.unrealized_net_pnl,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        relative
            .invested_capital_in_profit_share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.invested_profit,
                &c.cap,
                exit,
            )?;
        relative
            .invested_capital_in_loss_share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.invested_loss,
                &c.cap,
                exit,
            )?;
        relative
            .realized_cap_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &realized.cap.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        relative
            .net_pnl_change_1m_to_mcap
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned64>>(
                from,
                &realized.net_pnl.delta.absolute._1m.cents.height,
                all_market_cap,
                exit,
            )?;
        relative
            .net_pnl_change_1m_to_rcap
            .compute_binary::<_, _, Quotient<PartsPerMillionSigned64>>(
                from,
                &realized.net_pnl.delta.absolute._1m.cents.height,
                &c.cap,
                exit,
            )?;
        Ok(())
    }
}
