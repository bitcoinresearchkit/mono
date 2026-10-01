use bitview_cohort::CohortId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::{SatsToCents, SoprRatio};
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec,
};

use super::{
    ActivityMetrics, OutputMetrics, RealizedMetrics, Sources, SupplyMetrics, UnrealizedMetrics,
};
use crate::live::CohortState;

#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    pub supply: SupplyMetrics,
    pub outputs: OutputMetrics,
    pub activity: ActivityMetrics,
    pub realized: RealizedMetrics<M>,
    pub unrealized: UnrealizedMetrics,
    #[traversable(hidden)]
    sources: Sources<M>,
}

impl CohortMetrics {
    pub(crate) fn import(
        db: &Database,
        id: CohortId,
        version: Version,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let sources = Sources::import(db, id, version)?;
        let realized =
            RealizedMetrics::import(db, id, version, &sources, mappings, windows, prices)?;
        let unrealized = UnrealizedMetrics::new(id, version, &sources, &realized.price, mappings);
        Ok(Self {
            supply: SupplyMetrics::new(
                id, version, &sources, mappings, windows, prices, all_supply,
            ),
            outputs: OutputMetrics::new(id, version, &sources, mappings, windows),
            activity: ActivityMetrics::new(id, version, &sources, mappings, windows),
            realized,
            unrealized,
            sources,
        })
    }

    pub(crate) fn min_len(&self) -> usize {
        self.sources.min_len()
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.sources.stored_vecs_mut()
    }

    pub(crate) fn push(&mut self, state: &mut CohortState, price: Cents) {
        let unrealized = state.compute_unrealized_state(price);
        let realized = state.realized_block_data();
        let (unspent, spent) = state.output_counts();
        let (coindays, profit_sats, loss_sats) = state.core_activity();
        let sent = state.transfer_volume();
        let s = &mut self.sources;
        s.supply.push(state.supply_value());
        s.supply_in_profit.push(unrealized.supply_in_profit);
        s.supply_in_loss.push(unrealized.supply_in_loss);
        s.unspent_count.push(unspent);
        s.realized_cap.push(realized.cap);
        s.realized_price.push(realized.price());
        s.unrealized_profit.push(unrealized.unrealized_profit);
        s.unrealized_loss.push(unrealized.unrealized_loss);
        s.unrealized_net_pnl.push(CentsSigned::new(
            unrealized.unrealized_profit.inner() as i64 - unrealized.unrealized_loss.inner() as i64,
        ));
        s.spent_count.push_block(spent);
        s.transfer_sats.push_block(sent);
        s.transfer_cents.push_block(SatsToCents::apply(sent, price));
        s.profit_sats.push_block(profit_sats);
        s.profit_cents
            .push_block(SatsToCents::apply(profit_sats, price));
        s.loss_sats.push_block(loss_sats);
        s.loss_cents
            .push_block(SatsToCents::apply(loss_sats, price));
        s.coindays.push_block(coindays);
        s.realized_profit.push_block(realized.profit);
        s.realized_loss.push_block(realized.loss);
        s.realized_net_pnl.push_block(realized.net_pnl);
        s.value_destroyed.push_block(realized.value_destroyed);
    }

    pub(crate) fn compute_rest(&mut self, from: Height, exit: &Exit) -> Result<()> {
        self.realized.sopr.compute_binary::<_, _, SoprRatio>(
            from,
            &self.activity.transfer_volume.sum._24h.cents.height,
            &self.realized.value_destroyed.sum._24h.cents.height,
            exit,
        )
    }
}
