use bitview_cohort::{AgeRange, CreationCohorts};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CentsSquaredSats;
use bitview_traversable::Traversable;
use bitview_vecs::DisjointAgeSources;
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{NetUnrealizedByCohort, UnrealizedByCohort};
use crate::state::UnrealizedState;

#[derive(Traversable)]
pub struct UnrealizedVecs<M: StorageMode = Rw> {
    /// Unrealized profit of a UTXO cohort's unspent outputs: market value at
    /// the represented block minus creation-date value, summed where spot is
    /// above creation price.
    pub profit: UnrealizedByCohort<Cents, M>,
    /// Unrealized loss of a UTXO cohort's unspent outputs: creation-date value
    /// minus market value at the represented block, summed where spot is below
    /// creation price.
    pub loss: UnrealizedByCohort<Cents, M>,
    /// Net unrealized profit and loss of a UTXO cohort: unrealized profit
    /// minus unrealized loss.
    pub net_pnl: NetUnrealizedByCohort<M>,
    /// Exact squared-price products retained as disjoint accounting inputs.
    #[traversable(hidden)]
    pub capitalized_cap_in_profit_raw: DisjointAgeSources<CentsSquaredSats, M>,
    #[traversable(hidden)]
    pub capitalized_cap_in_loss_raw: DisjointAgeSources<CentsSquaredSats, M>,
}

impl UnrealizedVecs {
    pub fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Box<Self>> {
        let profit =
            UnrealizedByCohort::import(db, "unrealized_profit", version + Version::ONE, mappings)?;
        let loss =
            UnrealizedByCohort::import(db, "unrealized_loss", version + Version::ONE, mappings)?;
        let net_pnl = NetUnrealizedByCohort::import(db, version, mappings)?;
        let capitalized_cap_in_profit_raw =
            DisjointAgeSources::import(db, "capitalized_cap_in_profit_raw", version)?;
        let capitalized_cap_in_loss_raw =
            DisjointAgeSources::import(db, "capitalized_cap_in_loss_raw", version)?;
        Ok(Box::new(Self {
            profit,
            loss,
            net_pnl,
            capitalized_cap_in_profit_raw,
            capitalized_cap_in_loss_raw,
        }))
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: &CreationCohorts<UnrealizedState>) {
        let profit = cohort_values.map(|state| state.unrealized_profit);
        self.profit.stored.push(&profit);
        let loss = cohort_values.map(|state| state.unrealized_loss);
        self.loss.stored.push(&loss);
        self.net_pnl.stored.push(&cohort_values.map(|state| {
            CentsSigned::new(
                state.unrealized_profit.inner() as i64 - state.unrealized_loss.inner() as i64,
            )
        }));
        self.capitalized_cap_in_profit_raw
            .push_age(&AgeRange::from_fn(|id| {
                CentsSquaredSats::new(id.select(&cohort_values.age).capitalized_cap_in_profit_raw)
            }));
        self.capitalized_cap_in_loss_raw
            .push_age(&AgeRange::from_fn(|id| {
                CentsSquaredSats::new(id.select(&cohort_values.age).capitalized_cap_in_loss_raw)
            }));
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.profit.stored.stored_vecs_mut().collect::<Vec<_>>();
        vecs.extend(self.loss.stored.stored_vecs_mut());
        vecs.extend(self.net_pnl.stored.stored_vecs_mut());
        vecs.extend(self.capitalized_cap_in_profit_raw.collect_vecs_mut());
        vecs.extend(self.capitalized_cap_in_loss_raw.collect_vecs_mut());
        vecs
    }
}
