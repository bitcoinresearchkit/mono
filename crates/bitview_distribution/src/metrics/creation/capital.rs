use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{families::Fiat, metrics::CohortCapital, state::UnrealizedState};

#[derive(Deref, DerefMut, Traversable)]
pub struct CreationCapital<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    base: CohortCapital<M>,
    /// Capital in profit: creation-date USD value of the cohort's outputs whose creation price is
    /// at or below the represented block's spot price.
    #[traversable(lead = "Capital in profit: what holders paid for the coins now at or above their cost.")]
    pub in_profit: Fiat<Cents, M>,
    /// Capital in loss: creation-date USD value of the cohort's outputs whose creation price is
    /// above the represented block's spot price.
    #[traversable(lead = "Capital in loss: what holders paid for the coins now below their cost.")]
    pub in_loss: Fiat<Cents, M>,
}

impl CreationCapital {
    pub(super) fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_capital: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        // Split before rounding: each side's exact creation value, rounded once.
        let split_version = version + Version::ONE;
        Ok(Self {
            base: CohortCapital::import(db, name, version, mappings, window_starts, all_capital)?,
            in_profit: Fiat::import(db, &name("capital_in_profit"), split_version, mappings)?,
            in_loss: Fiat::import(db, &name("capital_in_loss"), split_version, mappings)?,
        })
    }

    #[inline(always)]
    pub(super) fn push(&mut self, total: Cents, profitability: &UnrealizedState) {
        self.base.push(total);
        self.in_profit.push(profitability.capital_in_profit);
        self.in_loss.push(profitability.capital_in_loss);
    }
}
