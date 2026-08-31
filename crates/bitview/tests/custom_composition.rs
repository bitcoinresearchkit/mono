#![cfg(not(any(
    feature = "bedrock",
    feature = "blocks",
    feature = "coinflow",
    feature = "cointime",
    feature = "distribution",
    feature = "inputs",
    feature = "mappings",
    feature = "mining",
    feature = "outputs",
    feature = "pools",
    feature = "price",
    feature = "transactions",
)))]

use bitview::{ComputePluginSet, Config, ImportContext, PluginSet, UpdateContext, run};
use bitview_plugin::ComputePlugin;
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use vecdb::{Rw, StorageMode};

#[derive(PluginSet, Traversable)]
struct Plugins<M: StorageMode = Rw> {
    indexer: Indexer<M>,
}

impl Plugins {
    fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        Ok(Self {
            indexer: Indexer::import(context, reader)?,
        })
    }
}

impl<M: StorageMode> HasIndexer<M> for Plugins<M> {
    fn indexer(&self) -> &Indexer<M> {
        &self.indexer
    }
}

impl ComputePluginSet for Plugins {
    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.indexer.compute((), context)
    }

    fn commit(&mut self) -> Result<()> {
        self.indexer.commit()
    }
}

fn custom_runner(config: Config, exit: Exit) -> Result<()> {
    run(config, exit, Plugins::import)
}

#[test]
fn runner_accepts_a_composition_without_default_plugins() {
    let _: fn(Config, Exit) -> Result<()> = custom_runner;
}
