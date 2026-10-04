#![doc = include_str!("../README.md")]

use std::{
    path::PathBuf,
    thread::{self, sleep},
    time::Duration,
};

use bitview_query::AsyncQuery;
use bitview_server::{Server, ServerConfig};
use brk_exit::Exit;
use brk_logger::init;
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::Client;
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
};
use tracing::info;
use vecdb::{Budgeted, ReadOnlyClone};

mod config;
mod paths;

pub use bitview_query::{Error, QueryPluginSet, Result};
pub use bitview_runtime::{
    BootstrapAction, ComputePluginSet, DEFAULT_CACHE_BUDGET, ImportContext, PluginSet,
    UpdateContext, bootstrap, update,
};
pub use config::Config;

/// Fully resolved settings for one Bitview runner.
pub struct RunConfig {
    /// Bitcoin Core RPC client.
    pub client: Client,
    /// Directory containing Bitcoin Core's block files.
    pub blocks_path: PathBuf,
    /// HTTP server and data-directory settings.
    pub server: ServerConfig,
    /// Bytes of the shared vector cache ([`DEFAULT_CACHE_BUDGET`] unless configured).
    pub cache_budget: usize,
}

/// Runs the Bitview daemon process with the supplied composition: reads the
/// configuration file and arguments, initializes logging and the shutdown handler,
/// then serves until exit.
pub fn run<P>(import: impl FnMut(ImportContext<'_>, &Reader) -> brk_error::Result<P>) -> Result<()>
where
    P: ComputePluginSet + ReadOnlyClone,
    P::ReadOnly: QueryPluginSet + 'static,
{
    let config = Config::import()?;

    init(Some(&config.server.logs_path()))?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    run_with(config, exit, import)
}

/// Runs one process-lifetime plugin composition with resolved settings and a
/// shutdown coordinator.
fn run_with<P>(
    config: RunConfig,
    exit: Exit,
    mut import: impl FnMut(ImportContext<'_>, &Reader) -> brk_error::Result<P>,
) -> Result<()>
where
    P: ComputePluginSet + ReadOnlyClone,
    P::ReadOnly: QueryPluginSet + 'static,
{
    let RunConfig {
        client,
        blocks_path,
        server,
        cache_budget,
    } = config;
    let reader = Reader::new(blocks_path, &client);
    let outputs_path = server.data_path.clone();
    Budgeted::init_global(cache_budget)?;
    let import_context = ImportContext::new(&outputs_path, &exit);
    let update_context = UpdateContext::new(&exit);

    // Outside the import lock; bootstrap measures its backlog against the synced tip.
    client.wait_for_synced_node()?;

    let mut plugins = bootstrap(
        import_context,
        |context| import(context, &reader),
        update_context,
    )?;

    let mut mempool = Mempool::new(&client);

    let query = AsyncQuery::build(&plugins, Some(mempool.read_only_clone()));

    // Updates close the plugin set's gate and reads wait on the indexer's: they must be one gate,
    // or queries would read state while it is being rewritten.
    if !query.sync(|q| q.reads_under(plugins.publication())) {
        return Err(Error::Internal(
            "the plugin set's publication is not its indexer's; queries would read unpublished state",
        ));
    }

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let server = runtime.block_on(Server::bind(&query, server))?;
    let server_handle = runtime.spawn(server.serve());

    let resolver = query.sync(|q| q.indexer_prevout_resolver());
    thread::spawn(move || {
        mempool.start_with(resolver);
    });

    info!("Waiting for new blocks...");

    loop {
        // Compare against published state, not a cached node observation: a
        // reorg can replace the tip without changing its height.
        while query.sync(|q| q.tip_blockhash()) == client.get_best_block_hash()? {
            if server_handle.is_finished() {
                return server_stopped(&runtime, server_handle);
            }
            sleep(Duration::from_secs(1));
        }

        // Wait with the publication gate open: queries keep reading the last published state.
        client.wait_for_synced_node()?;

        info!("Chain tip changed; updating...");

        update(&mut plugins, update_context)?;

        if server_handle.is_finished() {
            return server_stopped(&runtime, server_handle);
        }

        info!("Waiting for new blocks...");
    }
}

fn server_stopped(runtime: &Runtime, handle: JoinHandle<Result<()>>) -> Result<()> {
    runtime.block_on(handle)??;
    Err(Error::Internal("HTTP server stopped unexpectedly"))
}
