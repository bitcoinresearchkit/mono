use bitview::{BootstrapAction, QueryPluginSet};
use bitview_default::DefaultPlugins;
use bitview_plugin::{ComputePlugin, ImportContext, Publication, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution_profitability::{
    Dependencies, HasDistributionProfitability, Vecs as DistributionProfitability,
};
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use bitview_plugin_utxo_history::HasUtxoHistory;
use bitview_runtime::{ComputePluginSet, PluginSet};
use bitview_traversable::Traversable;
use bitviewd::run;
use brk_error::Result;
use brk_reader::Reader;
use vecdb::{ReadableCloneableVec, Ro, Rw, StorageMode};

#[derive(PluginSet, Traversable)]
struct Plugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    defaults: DefaultPlugins<M>,
    #[traversable(flatten)]
    distribution_profitability: DistributionProfitability<M>,
}

impl Plugins {
    fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        let defaults = DefaultPlugins::import(context, reader)?;
        let distribution_profitability = DistributionProfitability::import(
            context,
            defaults.mappings(),
            &defaults.blocks().lookback.window_starts(),
            &defaults.price().spot.cents.height.read_only_boxed_clone(),
        )?;
        Ok(Self {
            defaults,
            distribution_profitability,
        })
    }

    fn compute_profitability(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let history = self.defaults.utxo_history().reader(
            self.defaults.inputs().origins.spends(),
            &self.defaults.outputs().creations,
        )?;
        self.distribution_profitability.compute(
            Dependencies {
                history: &history,
                from: self.defaults.indexer().safe_lengths().height,
                prices: &self
                    .defaults
                    .price()
                    .spot
                    .cents
                    .height
                    .read_only_boxed_clone(),
                timestamps: &self
                    .defaults
                    .mappings()
                    .timestamp
                    .monotonic
                    .read_only_boxed_clone(),
            },
            context,
        )
    }
}

impl<M: StorageMode> HasDistributionProfitability<M> for Plugins<M> {
    fn distribution_profitability(&self) -> &DistributionProfitability<M> {
        &self.distribution_profitability
    }
}

impl QueryPluginSet for Plugins<Ro> {
    type Capabilities = DefaultPlugins<Ro>;
    fn query_capabilities(&self) -> &Self::Capabilities {
        &self.defaults
    }
}

impl ComputePluginSet for Plugins {
    fn publication(&self) -> &Publication {
        self.defaults.publication()
    }

    fn bootstrap_compute(&mut self, context: UpdateContext<'_>) -> Result<BootstrapAction> {
        self.defaults
            .bootstrap_compute(context)?
            .then_compute(|| self.compute_profitability(context))
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.defaults.compute(context)?;
        self.compute_profitability(context)
    }

    fn commit(&mut self) -> Result<()> {
        self.defaults.commit()
    }
}

fn main() -> Result<()> {
    run(Plugins::import)
}

#[cfg(test)]
mod tests {
    use std::thread;

    use bitview_plugin_distribution_profitability::ID;
    use bitview_query::{SeriesEntryLookup, Vecs as QueryVecs};
    use brk_rpc::{Auth, Client};
    use brk_types::Index;
    use tempfile::tempdir;
    use vecdb::Budgeted;

    use super::*;

    #[test]
    fn profitability_is_queryable_only_when_composed() {
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                Budgeted::init_global(2 * 1024 * 1024 * 1024).unwrap();
                let directory = tempdir().unwrap();
                let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
                let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
                let plugins =
                    Plugins::import(ImportContext::new(directory.path()), &reader).unwrap();
                let defaults = QueryVecs::build(&plugins.defaults);
                assert!(!matches!(
                    defaults.lookup_entry(
                        &"utxos_0pct_to_10pct_in_profit_supply_sats".into(),
                        Index::Height
                    ),
                    SeriesEntryLookup::Found(_)
                ));
                let optional = QueryVecs::build(&plugins);
                for name in [
                    "utxos_0pct_to_10pct_in_profit_supply_sats",
                    "utxos_0pct_to_10pct_in_loss_under_4m_realized_cap",
                    "utxos_0pct_to_10pct_in_profit_over_6m_nupl_ppm",
                ] {
                    let SeriesEntryLookup::Found(entry) =
                        optional.lookup_entry(&name.into(), Index::Height)
                    else {
                        panic!("missing optional series {name}");
                    };
                    assert_eq!(entry.plugin().id(), ID);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
