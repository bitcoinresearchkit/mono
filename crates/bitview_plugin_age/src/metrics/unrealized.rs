use bitview_cohort::{CohortContext, CohortId};
use bitview_distribution::families::Fiat;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::state::UnrealizedState;

#[derive(Traversable)]
pub struct UnrealizedVecs<M: StorageMode = Rw> {
    /// Unrealized profit of the cohort's unspent outputs: market value at the
    /// represented block minus creation-date value, summed where spot is above
    /// creation price.
    pub profit: Fiat<Cents, M>,
    /// Unrealized loss of the cohort's unspent outputs: creation-date value
    /// minus market value at the represented block, summed where spot is below
    /// creation price.
    pub loss: Fiat<Cents, M>,
    /// Net unrealized profit and loss of the cohort: unrealized profit minus
    /// unrealized loss.
    pub net_pnl: Fiat<CentsSigned, M>,
}

impl UnrealizedVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        Ok(Self {
            profit: Fiat::import(
                db,
                &name("unrealized_profit"),
                version + Version::ONE,
                mappings,
            )?,
            loss: Fiat::import(
                db,
                &name("unrealized_loss"),
                version + Version::ONE,
                mappings,
            )?,
            net_pnl: Fiat::import(db, &name("net_unrealized_pnl"), version, mappings)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &UnrealizedState) {
        self.profit.push(state.unrealized_profit);
        self.loss.push(state.unrealized_loss);
        self.net_pnl.push(CentsSigned::new(
            state.unrealized_profit.inner() as i64 - state.unrealized_loss.inner() as i64,
        ));
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [
            self.profit.stored_mut(),
            self.loss.stored_mut(),
            self.net_pnl.stored_mut(),
        ]
    }
}
