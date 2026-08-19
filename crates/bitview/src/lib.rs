#![doc = include_str!("../README.md")]

use brk_error::Result;

use std::{
    path::Path,
    thread::{self, sleep},
    time::{Duration, Instant},
};

pub use bitview_composition::DefaultPlugins;
use bitview_plugin_indexer::Indexer;
use bitview_query::AsyncQuery;
pub use bitview_query::QueryPluginSet;
pub use bitview_runtime::{BootstrapAction, ComputePluginSet, PluginSet, bootstrap, update};
use bitview_server::{Server, ServerConfig};
use brk_mempool::Mempool;
use brk_reader::Reader;
use tracing::info;
use vecdb::{Exit, ReadOnlyClone};

mod config;
mod paths;

use crate::{config::Config, paths::default_logs_dir};

/// The Bitview project website.
pub const HOMEPAGE: &str = "https://bitview.dev";

/// The official hosted Bitview instance.
pub const INSTANCE: &str = "https://bitview.space";

/// The toolkit Bitview is built on.
pub const TOOLKIT: &str = "https://bitcoinresearchkit.org";

/// Runs the default Bitview composition.
pub fn run() -> Result<()> {
    run_with(|outputs_path, reader| {
        let indexer = Indexer::import(outputs_path, reader)?;
        DefaultPlugins::forced_import(outputs_path, indexer)
    })
}

/// Runs Bitview with a statically composed built-in extension.
pub fn run_with<P>(mut import: impl FnMut(&Path, &Reader) -> Result<P>) -> Result<()>
where
    P: ComputePluginSet + ReadOnlyClone,
    P::ReadOnly: QueryPluginSet + 'static,
{
    let config = Config::import()?;

    brk_logger::init(Some(&default_logs_dir()))?;

    let client = config.rpc()?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let reader = Reader::new(config.blocksdir(), &client);
    let outputs_path = config.bitviewdir();

    client.wait_for_synced_node()?;

    let mut plugins = bootstrap(&outputs_path, || import(&outputs_path, &reader), &exit)?;

    let mempool = Mempool::new(&client);

    let query = AsyncQuery::build(&plugins, Some(mempool.clone()));

    let mempool_clone = mempool.clone();
    let resolver = query.sync(|q| q.indexer_prevout_resolver());
    thread::spawn(move || {
        mempool_clone.start_with(resolver);
    });

    let server_config = ServerConfig {
        data_path: outputs_path,
        website: config.website(),
        cdn_cache_mode: config.cdn_cache_mode(),
        max_weight: config.max_weight(),
        max_utxos: config.max_utxos(),
    };

    let port = config.bitviewport();
    let server_query = query.clone();

    let future = async move {
        let server = Server::new(&server_query, server_config);

        tokio::spawn(async move {
            server.serve(port).await.unwrap();
        });

        Ok(()) as Result<()>
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let _handle = runtime.spawn(future);

    let mut last_height = query.sync(|q| q.indexer().indexed_height());
    info!("Waiting for new blocks...");

    loop {
        while last_height == client.get_last_height()? {
            sleep(Duration::from_secs(1));
        }

        client.wait_for_synced_node()?;

        last_height = client.get_last_height()?;

        info!("{} blocks found.", u32::from(last_height) + 1);

        let total_start = Instant::now();

        update(&mut plugins, &exit)?;

        info!("Total time: {:?}", total_start.elapsed());
        info!("Waiting for new blocks...");
    }
}
