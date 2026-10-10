use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::{families::Fiat, metrics::CohortCapital};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, ReadableCloneableVec, Rw, StorageMode};

use crate::state::UnrealizedState;

#[derive(Deref, DerefMut, Traversable)]
pub struct CapitalVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: CohortCapital<M>,
    /// Capital in profit: the cohort's outputs whose creation price is less
    /// than or equal to the represented block's spot price.
    pub in_profit: Fiat<Cents, M>,
    /// Capital in loss: the cohort's outputs whose creation price is greater
    /// than the represented block's spot price.
    pub in_loss: Fiat<Cents, M>,
}

impl CapitalVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_capital: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        Ok(Self {
            base: CohortCapital::import(db, name, version, mappings, window_starts, all_capital)?,
            in_profit: Fiat::import(db, &name("capital_in_profit"), version, mappings)?,
            in_loss: Fiat::import(db, &name("capital_in_loss"), version, mappings)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, total: Cents, profitability: &UnrealizedState, price: Cents) {
        let (in_profit, in_loss) = profitability.capital_split(price);
        self.base.push(total);
        self.in_profit.push(in_profit);
        self.in_loss.push(in_loss);
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [
            self.base.stored_mut(),
            self.in_profit.stored_mut(),
            self.in_loss.stored_mut(),
        ]
    }
}
