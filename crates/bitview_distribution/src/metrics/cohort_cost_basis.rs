use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, Price, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

/// A cohort's mean creation price; `name` gives each series name from its metric
/// (`cost_basis_per_coin_avg` -> `utxos_1d_to_1w_old_cost_basis_per_coin_avg`).
#[derive(Traversable)]
pub struct CohortCostBasis<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by satoshis.
    per_coin: PerCoin,
    #[traversable(hidden)]
    stored: CachedSeries<Height, Cents, M>,
}

#[derive(Clone, Traversable)]
struct PerCoin {
    /// The weighted mean; zero while the cohort holds no supply.
    avg: Price<LazyPerBlock<Cents>>,
    /// Realized price: realized cap divided by supply, the weighted mean.
    realized_price: Price<LazyPerBlock<Cents>>,
}

impl CohortCostBasis {
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let stored = import_cached(db, &name("cost_basis_per_coin_avg_cents"), version)?;
        let price =
            |metric: &str| Price::from_height_source(&name(metric), version, &stored, mappings);
        Ok(Self {
            per_coin: PerCoin {
                avg: price("cost_basis_per_coin_avg"),
                realized_price: price("realized_price"),
            },
            stored,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, avg: Cents) {
        self.stored.push(avg);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
