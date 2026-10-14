use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::metrics::{CohortCostBasis, ShareTotals};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec,
};

use super::{
    ActivityMetrics, CapitalMetrics, OutputMetrics, RealizedMetrics, Sources, SupplyMetrics,
    UnrealizedMetrics,
};
use crate::live::CohortState;

#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    supply: SupplyMetrics,
    capital: CapitalMetrics,
    outputs: OutputMetrics,
    activity: ActivityMetrics,
    realized: RealizedMetrics,
    unrealized: UnrealizedMetrics,
    cost_basis: CohortCostBasis<M>,
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
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let sources = Sources::import(db, id, version)?;
        let cost_basis = CohortCostBasis::import(
            db,
            |metric| CohortContext::Utxo.metric_name(id, metric),
            version,
            mappings,
        )?;
        Ok(Self {
            supply: SupplyMetrics::new(
                id,
                version,
                &sources,
                mappings,
                windows,
                prices,
                totals.supply,
            ),
            capital: CapitalMetrics::new(id, version, &sources, mappings, windows, totals.capital),
            outputs: OutputMetrics::new(id, version, &sources, mappings, windows),
            activity: ActivityMetrics::new(id, version, &sources, mappings, windows),
            realized: RealizedMetrics::new(id, version, &sources, mappings, windows),
            unrealized: UnrealizedMetrics::new(id, version, &sources, mappings),
            cost_basis,
            sources,
        })
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.sources.stored_vecs_mut();
        vecs.push(self.cost_basis.stored_mut());
        vecs
    }

    pub(crate) fn push(&mut self, state: &mut CohortState, price: Cents) {
        let unrealized = state.compute_unrealized_state(price);
        let realized = state.realized_block_data();
        let (unspent, spent) = state.output_counts();
        let coindays = state.coindays_destroyed();
        let sent = state.transfer_volume();
        let s = &mut self.sources;
        s.supply.push(state.supply_value());
        s.supply_in_profit.push(unrealized.supply_in_profit);
        s.supply_in_loss.push(unrealized.supply_in_loss);
        s.unspent_count.push(unspent);
        s.realized_cap.push(realized.cap);
        let (capital_in_profit, capital_in_loss) = unrealized.capital_split(price);
        s.capital_in_profit.push(capital_in_profit);
        s.capital_in_loss.push(capital_in_loss);
        s.unrealized_profit.push(unrealized.unrealized_profit);
        s.unrealized_loss.push(unrealized.unrealized_loss);
        s.unrealized_net_pnl.push(CentsSigned::new(
            unrealized.unrealized_profit.inner() as i64 - unrealized.unrealized_loss.inner() as i64,
        ));
        s.spent_count.push_block(spent);
        s.transfer_sats.push_block(sent);
        s.transfer_cents.push_block(SatsToCents::apply(sent, price));
        s.coindays.push_block(coindays);
        s.realized_profit.push_block(realized.profit);
        s.realized_loss.push_block(realized.loss);
        s.realized_net_pnl.push_block(realized.net_pnl);
        s.value_destroyed.push_block(realized.value_destroyed);
        self.cost_basis.push(realized.price());
    }
}
