use crate::{columns::Columns, realized::Realized, supply::Supply, unrealized::Unrealized};
use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::{RatioCents, RatioCentsSignedCents, RatioDollars, RatioSats};
use bitview_traversable::Traversable;
use bitview_vecs::PercentPerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{
    Cents, Height, PartsPerMillion32, PartsPerMillionSigned32, PartsPerMillionSigned64, Sats,
    Version,
};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};
#[derive(Traversable)]
pub struct Relative<M: StorageMode = Rw> {
    pub supply_dominance: PercentPerBlock<PartsPerMillion32, M>,
    pub supply_in_profit_share: PercentPerBlock<PartsPerMillion32, M>,
    pub supply_in_loss_share: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_mcap: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_mcap: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_own_mcap: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_own_mcap: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_profit_to_own_gross_pnl: PercentPerBlock<PartsPerMillion32, M>,
    pub unrealized_loss_to_own_gross_pnl: PercentPerBlock<PartsPerMillion32, M>,
    pub net_unrealized_pnl_to_own_gross_pnl: PercentPerBlock<PartsPerMillionSigned32, M>,
    pub invested_capital_in_profit_share: PercentPerBlock<PartsPerMillion32, M>,
    pub invested_capital_in_loss_share: PercentPerBlock<PartsPerMillion32, M>,
    pub realized_cap_to_own_mcap: PercentPerBlock<PartsPerMillion32, M>,
    pub net_pnl_change_1m_to_mcap: PercentPerBlock<PartsPerMillionSigned64, M>,
    pub net_pnl_change_1m_to_rcap: PercentPerBlock<PartsPerMillionSigned64, M>,
}
impl Relative {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        mappings: &Mappings,
    ) -> Result<Self> {
        Ok(Self {
            supply_dominance: PercentPerBlock::forced_import(
                db,
                &id.metric_name("supply_dominance"),
                v,
                mappings,
            )?,
            supply_in_profit_share: PercentPerBlock::forced_import(
                db,
                &id.metric_name("supply_in_profit_share"),
                v,
                mappings,
            )?,
            supply_in_loss_share: PercentPerBlock::forced_import(
                db,
                &id.metric_name("supply_in_loss_share"),
                v,
                mappings,
            )?,
            unrealized_profit_to_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_profit_to_mcap"),
                v,
                mappings,
            )?,
            unrealized_loss_to_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_loss_to_mcap"),
                v,
                mappings,
            )?,
            unrealized_profit_to_own_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_profit_to_own_mcap"),
                v,
                mappings,
            )?,
            unrealized_loss_to_own_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_loss_to_own_mcap"),
                v,
                mappings,
            )?,
            unrealized_profit_to_own_gross_pnl: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_profit_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            unrealized_loss_to_own_gross_pnl: PercentPerBlock::forced_import(
                db,
                &id.metric_name("unrealized_loss_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            net_unrealized_pnl_to_own_gross_pnl: PercentPerBlock::forced_import(
                db,
                &id.metric_name("net_unrealized_pnl_to_own_gross_pnl"),
                v,
                mappings,
            )?,
            invested_capital_in_profit_share: PercentPerBlock::forced_import(
                db,
                &id.metric_name("invested_capital_in_profit_share"),
                v,
                mappings,
            )?,
            invested_capital_in_loss_share: PercentPerBlock::forced_import(
                db,
                &id.metric_name("invested_capital_in_loss_share"),
                v,
                mappings,
            )?,
            realized_cap_to_own_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("realized_cap_to_own_mcap"),
                v,
                mappings,
            )?,
            net_pnl_change_1m_to_mcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("net_pnl_change_1m_to_mcap"),
                v,
                mappings,
            )?,
            net_pnl_change_1m_to_rcap: PercentPerBlock::forced_import(
                db,
                &id.metric_name("net_pnl_change_1m_to_rcap"),
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
        realized: &Realized,
        unrealized: &Unrealized,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.supply_dominance
            .compute_binary::<_, _, RatioSats<PartsPerMillion32>>(
                from, &c.supply, all_supply, exit,
            )?;
        self.supply_in_profit_share
            .compute_binary::<_, _, RatioSats<PartsPerMillion32>>(
                from,
                &c.supply_profit,
                &c.supply,
                exit,
            )?;
        self.supply_in_loss_share
            .compute_binary::<_, _, RatioSats<PartsPerMillion32>>(
                from,
                &c.supply_loss,
                &c.supply,
                exit,
            )?;
        self.unrealized_profit_to_mcap
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.unrealized_profit,
                all_market_cap,
                exit,
            )?;
        self.unrealized_loss_to_mcap
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.unrealized_loss,
                all_market_cap,
                exit,
            )?;
        self.unrealized_profit_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &unrealized.profit.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        self.unrealized_loss_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &unrealized.loss.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        self.unrealized_profit_to_own_gross_pnl
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.unrealized_profit,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        self.unrealized_loss_to_own_gross_pnl
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.unrealized_loss,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        self.net_unrealized_pnl_to_own_gross_pnl
            .compute_binary::<_, _, RatioCentsSignedCents<PartsPerMillionSigned32>>(
                from,
                &c.unrealized_net_pnl,
                &c.unrealized_gross_pnl,
                exit,
            )?;
        self.invested_capital_in_profit_share
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.invested_profit,
                &c.cap,
                exit,
            )?;
        self.invested_capital_in_loss_share
            .compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &c.invested_loss,
                &c.cap,
                exit,
            )?;
        self.realized_cap_to_own_mcap
            .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                from,
                &realized.cap.usd.height,
                &supply.total.usd.height,
                exit,
            )?;
        self.net_pnl_change_1m_to_mcap
            .compute_binary::<_, _, RatioCentsSignedCents<PartsPerMillionSigned64>>(
                from,
                &realized.net_pnl.delta.absolute._1m.cents.height,
                all_market_cap,
                exit,
            )?;
        self.net_pnl_change_1m_to_rcap
            .compute_binary::<_, _, RatioCentsSignedCents<PartsPerMillionSigned64>>(
                from,
                &realized.net_pnl.delta.absolute._1m.cents.height,
                &c.cap,
                exit,
            )?;
        Ok(())
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
