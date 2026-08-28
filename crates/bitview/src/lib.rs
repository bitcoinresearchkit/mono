#![doc = include_str!("../README.md")]

use std::{
    path::PathBuf,
    thread::{self, sleep},
    time::Duration,
};

use bitview_query::AsyncQuery;
pub use bitview_query::QueryPluginSet;
pub use bitview_runtime::{
    BootstrapAction, ComputePluginSet, ImportContext, PluginSet, UpdateContext, bootstrap, update,
};
use bitview_server::{Server, ServerConfig};
use brk_error::{Error, Result};
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::Client;
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
};
use tracing::info;
use vecdb::{Exit, ReadOnlyClone};

/// The Bitview project website.
pub const HOMEPAGE: &str = "https://bitview.dev";

/// The official hosted Bitview instance.
pub const INSTANCE: &str = "https://bitview.space";

/// The toolkit Bitview is built on.
pub const TOOLKIT: &str = "https://bitcoinresearchkit.org";

/// Fully resolved settings for one Bitview runner.
pub struct Config {
    /// Bitcoin Core RPC client.
    pub client: Client,
    /// Directory containing Bitcoin Core's block files.
    pub blocks_path: PathBuf,
    /// HTTP server and data-directory settings.
    pub server: ServerConfig,
}

/// Runs one process-lifetime plugin composition with resolved settings and a
/// shutdown coordinator.
pub fn run<P>(
    config: Config,
    exit: Exit,
    mut import: impl FnMut(ImportContext<'_>, &Reader) -> Result<P>,
) -> Result<()>
where
    P: ComputePluginSet + ReadOnlyClone,
    P::ReadOnly: QueryPluginSet + 'static,
{
    let Config {
        client,
        blocks_path,
        server,
    } = config;
    let reader = Reader::new(blocks_path, &client);
    let outputs_path = server.data_path.clone();
    let import_context = ImportContext::new(&outputs_path);
    let update_context = UpdateContext::new(&exit);

    client.wait_for_synced_node()?;

    let mut plugins = bootstrap(
        import_context,
        |context| import(context, &reader),
        update_context,
    )?;

    let mempool = Mempool::new(&client);

    let query = AsyncQuery::build(&plugins, Some(mempool.clone()));

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let server = runtime.block_on(Server::bind(&query, server))?;
    let server_handle = runtime.spawn(server.serve());

    let mempool_clone = mempool.clone();
    let resolver = query.sync(|q| q.indexer_prevout_resolver());
    thread::spawn(move || {
        mempool_clone.start_with(resolver);
    });

    let mut last_height = query.sync(|q| q.indexer().indexed_height());
    info!("Waiting for new blocks...");

    loop {
        while last_height == client.get_last_height()? {
            if server_handle.is_finished() {
                return server_stopped(&runtime, server_handle);
            }
            sleep(Duration::from_secs(1));
        }

        client.wait_for_synced_node()?;

        last_height = client.get_last_height()?;

        info!("New chain tip: block {last_height}");

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
