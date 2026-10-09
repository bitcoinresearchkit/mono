use bitview::{QueryPluginSet, run};
use bitview_default::DefaultPlugins;
use bitview_plugin::{ComputePlugin, ImportContext, Publication, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use bitview_plugin_profitability::{Dependencies, HasProfitability, Vecs as Profitability};
use bitview_plugin_utxo_set::HasUtxoSet;
use bitview_runtime::{BootstrapAction, ComputePluginSet, PluginSet};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_reader::Reader;
use vecdb::{ReadableCloneableVec, Ro, Rw, StorageMode};

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

#[derive(PluginSet, Traversable)]
struct Plugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    defaults: DefaultPlugins<M>,
    #[plugin_set(has = HasProfitability<M>)]
    profitability: Profitability<M>,
}

impl Plugins {
    fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        let defaults = DefaultPlugins::import(context, reader)?;
        let profitability = Profitability::import(
            context,
            defaults.mappings(),
            &defaults.blocks().lookback.window_starts(),
            &defaults.price().spot.cents.height.read_only_boxed_clone(),
        )?;
        Ok(Self {
            defaults,
            profitability,
        })
    }

    fn compute_profitability(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let history = self.defaults.utxo_set().reader(
            self.defaults.inputs().origins.spends(),
            &self.defaults.outputs().creations,
        )?;
        self.profitability.compute(
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

    fn commit(&mut self, complete: bool) -> Result<()> {
        self.defaults.commit(complete)
    }
}

fn main() -> bitview::Result<()> {
    run(Plugins::import)
}
