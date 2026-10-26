use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use derive_more::{Deref, DerefMut};

use crate::{families::Supply, metrics::CohortSupply, state::UnrealizedState};

#[derive(Deref, DerefMut, Traversable)]
pub struct CreationSupply<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    base: CohortSupply<M>,
    /// Unspent supply in profit: the cohort's outputs whose creation price is
    /// less than or equal to the represented block's spot price.
    #[traversable(lead = "Supply in profit: coins whose price now is at or above the one they last moved at.")]
    pub in_profit: Supply<M>,
    /// Unspent supply in loss: the cohort's outputs whose creation price is
    /// greater than the represented block's spot price.
    #[traversable(lead = "Supply in loss: coins whose price now is below the one they last moved at.")]
    pub in_loss: Supply<M>,
}

impl CreationSupply {
    pub(super) fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        Ok(Self {
            base: CohortSupply::import(
                db,
                &name("supply"),
                version,
                mappings,
                window_starts,
                spot_price,
                all_supply,
            )?,
            in_profit: Supply::import(
                db,
                &name("supply_in_profit"),
                version,
                mappings,
                spot_price,
            )?,
            in_loss: Supply::import(db, &name("supply_in_loss"), version, mappings, spot_price)?,
        })
    }

    #[inline(always)]
    pub(super) fn push(&mut self, total: Sats, profitability: &UnrealizedState) {
        self.base.total.push(total);
        self.in_profit.push(profitability.supply_in_profit);
        self.in_loss.push(profitability.supply_in_loss);
    }
}
