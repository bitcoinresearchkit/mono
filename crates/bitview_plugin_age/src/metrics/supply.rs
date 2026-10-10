use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::{families::Supply, metrics::CohortSupply};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use derive_more::{Deref, DerefMut};

use crate::state::UnrealizedState;

#[derive(Deref, DerefMut, Traversable)]
pub struct SupplyVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: CohortSupply<M>,
    /// Unspent supply in profit: the cohort's outputs whose creation price is
    /// less than or equal to the represented block's spot price.
    pub in_profit: Supply<M>,
    /// Unspent supply in loss: the cohort's outputs whose creation price is
    /// greater than the represented block's spot price.
    pub in_loss: Supply<M>,
}

impl SupplyVecs {
    pub fn import(
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
    pub fn push(&mut self, total: Sats, profitability: &UnrealizedState) {
        self.base.total.push(total);
        self.in_profit.push(profitability.supply_in_profit);
        self.in_loss.push(profitability.supply_in_loss);
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [
            self.base.total.stored_mut(),
            self.in_profit.stored_mut(),
            self.in_loss.stored_mut(),
        ]
    }
}
