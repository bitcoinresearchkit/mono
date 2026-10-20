//! Metric bundles for age, halving-epoch, creation-year and entry-price cohorts. The owning plugin
//! classifies and replays.

mod activity;
mod capital;
mod realized;
mod supply;
mod unrealized;

pub use activity::CreationActivity;
pub use capital::CreationCapital;
pub use realized::CreationRealized;
pub use supply::CreationSupply;
pub use unrealized::CreationUnrealized;

use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use super::{CohortCostBasis, ShareTotals, StoredUtxos};
use crate::state::{MappedUTXOCohortState, RealizedOps};

#[derive(Traversable)]
pub struct CreationCohort<M: StorageMode = Rw> {
    pub supply: CreationSupply<M>,
    pub capital: CreationCapital<M>,
    pub utxos: StoredUtxos<M>,
    pub activity: CreationActivity<M>,
    pub realized: CreationRealized<M>,
    pub unrealized: CreationUnrealized<M>,
    cost_basis: CohortCostBasis<M>,
}

impl CreationCohort {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        Ok(Self {
            supply: CreationSupply::import(
                db,
                cohort,
                version,
                mappings,
                window_starts,
                spot_price,
                totals.supply,
            )?,
            capital: CreationCapital::import(
                db,
                cohort,
                version,
                mappings,
                window_starts,
                totals.capital,
            )?,
            utxos: StoredUtxos::import(db, name, version, mappings, window_starts)?,
            activity: CreationActivity::import(db, cohort, version, mappings, window_starts)?,
            realized: CreationRealized::import(db, cohort, version, mappings, window_starts)?,
            unrealized: CreationUnrealized::import(db, cohort, version, mappings)?,
            cost_basis: CohortCostBasis::import(db, name, version, mappings)?,
        })
    }

    /// Appends the block's values at the block's spot `price`.
    #[inline(always)]
    pub fn push<R: RealizedOps>(&mut self, state: &mut MappedUTXOCohortState<R>, price: Cents) {
        let profitability = state.compute_unrealized_state(price);
        let realized = state.realized_block_data();
        self.supply.push(state.supply.value, &profitability);
        self.capital.push(realized.cap, &profitability);
        self.unrealized.push(&profitability);
        self.utxos.push(state.output_counts());
        self.activity
            .push(state.sent, state.coindays_destroyed(), price);
        self.realized.push(&realized);
        self.cost_basis.push(realized.price());
    }
}
