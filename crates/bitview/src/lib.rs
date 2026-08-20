#![doc = include_str!("../README.md")]

use brk_error::Result;

use std::{
    path::PathBuf,
    thread::{self, sleep},
    time::{Duration, Instant},
};

use bitview_query::AsyncQuery;
pub use bitview_query::QueryPluginSet;
pub use bitview_runtime::{
    BootstrapAction, ComputePluginSet, ImportContext, PluginSet, UpdateContext, bootstrap, update,
};
use bitview_server::{Server, ServerConfig};
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::Port;
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
    /// HTTP listener port, or the server default when omitted.
    pub port: Option<Port>,
    /// HTTP server and data-directory settings.
    pub server: ServerConfig,
}

/// Runs a plugin composition with resolved settings and a shutdown coordinator.
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
        port,
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

    let mempool_clone = mempool.clone();
    let resolver = query.sync(|q| q.indexer_prevout_resolver());
    thread::spawn(move || {
        mempool_clone.start_with(resolver);
    });

    let server_query = query.clone();

    let future = async move {
        let server = Server::new(&server_query, server);

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

        update(&mut plugins, update_context)?;

        info!("Total time: {:?}", total_start.elapsed());
        info!("Waiting for new blocks...");
    }
}
