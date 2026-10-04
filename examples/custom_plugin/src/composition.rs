use bitview::QueryPluginSet;
use bitview_default::DefaultPlugins;
use bitview_plugin::{ComputePlugin, ImportContext, Publication, UpdateContext};
use bitview_plugin_indexer::HasIndexer;
use bitview_runtime::{BootstrapAction, ComputePluginSet, PluginSet};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_reader::Reader;
use vecdb::{Ro, Rw, StorageMode};

use crate::near_full_blocks::{Dependencies, Vecs as NearFullBlocks};

#[derive(PluginSet, Traversable)]
pub struct Plugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    defaults: DefaultPlugins<M>,
    near_full_blocks: NearFullBlocks<M>,
}

impl Plugins {
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        let defaults = DefaultPlugins::import(context, reader)?;
        let near_full_blocks = NearFullBlocks::import(context)?;

        Ok(Self {
            defaults,
            near_full_blocks,
        })
    }

    fn compute_near_full_blocks(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let indexer = self.defaults.indexer();
        self.near_full_blocks.compute(
            Dependencies {
                safe_height: indexer.safe_lengths().height,
                block_weights: &indexer.vecs().blocks.weight,
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
            .then_compute(|| self.compute_near_full_blocks(context))
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.defaults.compute(context)?;
        self.compute_near_full_blocks(context)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        self.defaults.commit(complete)
    }
}
