//! The metric bundle of a creation cohort, whose coins are classified once, when created: by
//! age, halving epoch, creation year or entry price. The owning plugin classifies and replays.

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

use super::{CohortCostBasis, ShareTotals, StoredOutputs};
use crate::state::{MappedUTXOCohortState, RealizedOps};

/// One creation cohort's series.
#[derive(Traversable)]
pub struct CreationCohort<M: StorageMode = Rw> {
    pub supply: CreationSupply<M>,
    pub capital: CreationCapital<M>,
    pub outputs: StoredOutputs<M>,
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
            outputs: StoredOutputs::import(db, name, version, mappings, window_starts)?,
            activity: CreationActivity::import(db, cohort, version, mappings, window_starts)?,
            realized: CreationRealized::import(db, cohort, version, mappings, window_starts)?,
            unrealized: CreationUnrealized::import(db, cohort, version, mappings)?,
            cost_basis: CohortCostBasis::import(db, name, version, mappings)?,
        })
    }

    /// Writes the block's values, with the cohort's profitability at `price`; returns its
    /// capital.
    #[inline(always)]
    pub fn push<R: RealizedOps>(
        &mut self,
        state: &mut MappedUTXOCohortState<R>,
        price: Cents,
    ) -> Cents {
        let profitability = state.compute_unrealized_state(price);
        let realized = state.realized_block_data();
        self.supply.push(state.supply_value(), &profitability);
        self.capital.push(realized.cap, &profitability);
        self.unrealized.push(&profitability);
        self.outputs.push(state.output_counts());
        self.activity
            .push(state.transfer_volume(), state.coindays_destroyed(), price);
        self.realized.push(&realized);
        self.cost_basis.push(realized.price());
        realized.cap
    }
}
