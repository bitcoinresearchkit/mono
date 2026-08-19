use std::path::Path;

use bitview::{BootstrapAction, ComputePluginSet, DefaultPlugins, PluginSet, QueryPluginSet};
use bitview_plugin::{ComputePlugin, Plugin};
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_reader::Reader;
use vecdb::{Exit, Ro, Rw, StorageMode};

use crate::near_full_blocks::{Dependencies, HasNearFullBlocks, Vecs as NearFullBlocks};

#[derive(Traversable)]
pub struct Plugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    defaults: DefaultPlugins<M>,
    near_full_blocks: NearFullBlocks<M>,
}

impl Plugins {
    pub fn forced_import(outputs_path: &Path, reader: &Reader) -> Result<Self> {
        let indexer = Indexer::import(outputs_path, reader)?;
        let defaults = DefaultPlugins::forced_import(outputs_path, indexer)?;
        let near_full_blocks = NearFullBlocks::forced_import(outputs_path)?;

        Ok(Self {
            defaults,
            near_full_blocks,
        })
    }

    fn compute_near_full_blocks(&mut self, exit: &Exit) -> Result<()> {
        self.near_full_blocks.compute(
            Dependencies {
                indexer: self.defaults.indexer(),
            },
            exit,
        )
    }
}

impl<M: StorageMode> HasNearFullBlocks<M> for Plugins<M> {
    fn near_full_blocks(&self) -> &NearFullBlocks<M> {
        &self.near_full_blocks
    }
}

impl PluginSet for Plugins {
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin)) {
        self.defaults.for_each_plugin(visit);
        visit(self.near_full_blocks());
    }
}

impl PluginSet for Plugins<Ro> {
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin)) {
        self.defaults.for_each_plugin(visit);
        visit(self.near_full_blocks());
    }
}

impl QueryPluginSet for Plugins<Ro> {
    type Capabilities = DefaultPlugins<Ro>;

    fn query_capabilities(&self) -> &Self::Capabilities {
        &self.defaults
    }
}

impl ComputePluginSet for Plugins {
    fn bootstrap_compute(&mut self, exit: &Exit) -> Result<BootstrapAction> {
        let action = self.defaults.bootstrap_compute(exit)?;
        self.compute_near_full_blocks(exit)?;
        Ok(action)
    }

    fn compute(&mut self, exit: &Exit) -> Result<()> {
        self.defaults.compute(exit)?;
        self.compute_near_full_blocks(exit)
    }

    fn prepare_publish(&mut self) -> Result<()> {
        self.defaults.prepare_publish()
    }
}
