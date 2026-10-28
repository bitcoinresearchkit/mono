#![doc = include_str!("../README.md")]

#[cfg(feature = "price")]
use std::thread;
use std::{
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview_query::AsyncQuery;
use bitview_server::{Server, ServerConfig};
use brk_exit::Exit;
use brk_logger::init;
#[cfg(feature = "price")]
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::Client;
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
};
use tracing::{info, warn};
use vecdb::{Budgeted, ReadOnlyClone};

mod bootstrap;
mod config;
mod paths;
mod update;

pub use bitview_plugin::{ImportContext, UpdateContext};
pub use bitview_query::{Error, QueryPluginSet, Result};
pub use bootstrap::bootstrap;
pub use config::{Config, DEFAULT_CACHE_BUDGET};

use bitview_plugin::ComputePluginSet;
use update::update;

/// Fully resolved settings for one Bitview runner.
pub struct RunConfig {
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
    raise_fd_limit();

    let exit = Exit::new();
    exit.install_signal_handlers();

    run_with(config, exit, import)
}

/// Raises the process's open-file soft limit for the daemon's stores and block
/// files. Only the soft limit, clamped to the hard limit: raising the hard limit
/// requires `CAP_SYS_RESOURCE` and would fail on containers and unprivileged
/// macOS processes.
pub fn raise_fd_limit() {
    const TARGET_NOFILE: libc::rlim_t = 15_000;
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `getrlimit` fills the `rlimit` it is given.
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) } != 0 {
        return;
    }
    let new_soft = limit.rlim_cur.max(TARGET_NOFILE).min(limit.rlim_max);
    if new_soft > limit.rlim_cur {
        limit.rlim_cur = new_soft;
        // SAFETY: `setrlimit` only reads the `rlimit` it is given.
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limit) } != 0 {
            warn!(
                "failed to raise NOFILE rlimit: {}",
                std::io::Error::last_os_error()
            );
        }
    }
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
    let reader = Reader::new(blocks_path, &client)?;
    reader.check_network()?;
    Budgeted::init_global(cache_budget)?;
    let import_context = ImportContext::new(&server.data_path, &exit);
    let update_context = UpdateContext::new(&exit);

    // Outside the import lock; bootstrap measures its backlog against the synced tip.
    client.wait_for_synced_node()?;

    let mut plugins = bootstrap(
        import_context,
        |context| import(context, &reader),
        update_context,
    )?;

    // Only the chain and price reads use the mempool: without them no driver runs.
    #[cfg(feature = "price")]
    let mut mempool = Mempool::new(&client);
    #[cfg(feature = "price")]
    let query = AsyncQuery::build(&plugins, Some(mempool.read_only_clone()));
    #[cfg(not(feature = "price"))]
    let query = AsyncQuery::build(&plugins, None);

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let server = runtime.block_on(Server::bind(&query, server))?;
    let server_handle = runtime.spawn(server.serve());

    #[cfg(feature = "price")]
    let published = {
        let resolver = query.sync(|q| q.indexer_prevout_resolver());
        let chain = query.sync(bitview_query::Query::clone);
        let published = mempool.chain_waker();
        thread::spawn(move || {
            mempool.start_with(resolver, || chain.tip_blockhash());
        });
        published
    };

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

        info!("Chain tip changed; updating...");

        let start = Instant::now();
        update(&mut plugins, update_context)?;
        let (height, hash) = query
            .sync(|q| q.tip_block())
            .expect("an update publishes at least genesis");
        info!(
            "Updated to block {height} {hash} in {:.2?}",
            start.elapsed()
        );
        #[cfg(feature = "price")]
        published.published();

        if server_handle.is_finished() {
            return server_stopped(&runtime, server_handle);
        }
    }
}

fn server_stopped(runtime: &Runtime, handle: JoinHandle<Result<()>>) -> Result<()> {
    runtime.block_on(handle)??;
    Err(Error::Internal("HTTP server stopped unexpectedly"))
}
