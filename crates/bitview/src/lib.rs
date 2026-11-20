#![doc = include_str!("../README.md")]

#[cfg(feature = "full-api")]
use std::{sync::mpsc::sync_channel, thread};
use std::{
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, Instant},
};

use bitview_query::AsyncQuery;
use bitview_server::{Server, ServerConfig};
use brk_exit::Exit;
use brk_logger::init;
#[cfg(feature = "full-api")]
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::Client;
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
};
use tracing::{debug, info, warn};
use vecdb::{Budgeted, ReadOnlyClone};

mod bootstrap;
mod config;
mod paths;
mod update;

pub use bitview_plugin::{ImportContext, UpdateContext};
pub use bitview_query::{Error, QueryPluginSet, Result};
pub use bitview_server::set_build;
pub use bootstrap::bootstrap;
pub use config::{Config, DEFAULT_CACHE_BUDGET};

use bitview_plugin::ComputePluginSet;
use update::update;

/// How often an idle server reads what the next update will: an update a minute after the last
/// still finds it in memory, one ten minutes after mostly on disk.
#[cfg(feature = "full-api")]
const WARM_INTERVAL: Duration = Duration::from_secs(30);

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

    let result = run_with(config, exit.clone(), import);
    // Past bootstrap, detached threads keep the databases open, so nothing dropped on the way
    // here joined their background work (the indexer's ingestion, a compaction): returning now
    // would end the process inside its critical section.
    exit.close();
    result
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
    #[cfg(feature = "full-api")]
    let mut mempool = Mempool::new(&client);
    #[cfg(feature = "full-api")]
    let mut audits = mempool.keep_audits(&server.data_path.join("mempool/audits"), &exit)?;
    // The RBF trees, removal reasons and first sightings outlive restarts: saved every few
    // minutes, and the last publication's at shutdown.
    #[cfg(feature = "full-api")]
    {
        let graveyard = mempool.keep_graveyard(server.data_path.join("mempool").join("graveyard"));
        exit.register_cleanup(move || graveyard.save());
    }
    #[cfg(feature = "full-api")]
    let query = AsyncQuery::build(&plugins, Some(mempool.read_only_clone()));
    #[cfg(not(feature = "full-api"))]
    let query = AsyncQuery::build(&plugins, None);

    #[cfg(feature = "full-api")]
    query.sync(bitview_query::Query::verify_block_positions)?;

    // Try to publish the orphan list before serving. If it fails, affected rows use live caching
    // until a complete scan is published.
    #[cfg(feature = "full-api")]
    query.sync(refresh_stale_blocks);
    #[cfg(feature = "full-api")]
    query.sync(|q| q.refresh_audits(None))?;

    record_node(&client, &query)?;
    let (data_path, logs_path) = (server.data_path.clone(), server.logs_path());
    record_data_bytes(&query, &data_path, &logs_path)?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let server = runtime.block_on(Server::bind(&query, server))?;
    let server_handle = runtime.spawn(server.serve());

    #[cfg(feature = "full-api")]
    let pool = mempool.read_only_clone();
    #[cfg(feature = "full-api")]
    let published = {
        // The indexer first; what it can't give (a parent above its tip, or a script it reads
        // from a block file that won't read), Core's UTXO set.
        let indexer = query.sync(|q| q.indexer_prevout_resolver());
        let node = brk_mempool::rpc_resolver(client.clone());
        let resolver = move |holes: &[_]| {
            let mut resolved = indexer(holes);
            let rest: Vec<_> = holes
                .iter()
                .filter(|hole| !resolved.contains_key(*hole))
                .copied()
                .collect();
            resolved.extend(node(&rest));
            resolved
        };
        let chain = query.sync(bitview_query::Query::clone);
        let published = mempool.chain_waker();
        thread::spawn(move || {
            let facts = chain.clone();
            let tip_facts = move |tip: &_| {
                facts
                    .tip_facts(tip)
                    .inspect_err(|error| warn!(%error, %tip, "block template: the chain's facts at the tip"))
                    .ok()
            };
            mempool.start_with(resolver, || chain.tip_blockhash(), tip_facts);
        });
        published
    };
    // On its own thread: a block arriving meanwhile never waits on the node's answers.
    #[cfg(feature = "full-api")]
    let stale_blocks = {
        let query = query.sync(bitview_query::Query::clone);
        let (refresh, wanted) = sync_channel(1);
        thread::spawn(move || {
            for () in wanted {
                refresh_stale_blocks(&query);
            }
        });
        refresh
    };

    info!("Waiting for new blocks...");

    #[cfg(feature = "full-api")]
    let mut warmed = Instant::now();
    // The node's tip the last update ran for: retrying it logs nothing new.
    let mut updating_to = None;
    #[cfg(feature = "full-api")]
    let mut capture = None;
    loop {
        if server_handle.is_finished() {
            return server_stopped(&runtime, server_handle);
        }
        // Compare against published state, not a cached node observation: a
        // reorg can replace the tip without changing its height.
        let best = client.get_best_block_hash()?;
        if query.sync(|q| q.tip_blockhash()) != best {
            if updating_to.replace(best) != Some(best) {
                #[cfg(feature = "full-api")]
                { capture = pool.capture_audit(); }
                info!("Chain tip changed; updating...");
                record_node(&client, &query)?;
            }
            let start = Instant::now();
            // One that moves nothing (a block the reader can't read yet, or ever: it logged why)
            // retries after the idle wait.
            if update(&mut plugins, update_context)? {
                let (height, hash) = query
                    .sync(|q| q.tip_block())
                    .expect("an update publishes at least genesis");
                info!(
                    "Updated to block {height} {hash} in {:.2?}",
                    start.elapsed()
                );
                #[cfg(feature = "full-api")]
                published.published();
                // Full, a read not yet started is due: it will see this publication.
                #[cfg(feature = "full-api")]
                let _ = stale_blocks.try_send(());

                #[cfg(feature = "full-api")]
                {
                    if let Err(error) = query.sync(|q| q.refresh_audits(None)) {
                        warn!(%error, "block audits: canonical view refresh failed");
                    }
                    let audit_start = Instant::now();
                    if let Some(capture) = capture.take() {
                        let age = capture.age;
                        match query.sync(|q| q.indexed_audit_block(best)) {
                            Ok(block) => if let Some(audit) = capture.compare(&block) {
                                let committed = audits.commit(&audit).or_else(|error| {
                                    warn!(%error, hash = %best, "block audit write failed; verifying pair before retry");
                                    audits.commit(&audit)
                                });
                                match committed {
                                    Ok(_) => {
                                        if let Err(error) = query.sync(|q| q.refresh_audits(Some(audit.summary))) {
                                            warn!(%error, hash = %best, "block audit canonical publication failed");
                                        }
                                        info!(hash = %best, ?age, elapsed = ?audit_start.elapsed(), "Block audit committed");
                                    }
                                    Err(error) => warn!(%error, hash = %best, "block audit commit failed"),
                                }
                            } else { info!(hash = %best, "Block audit skipped: capture parent differs"); },
                            Err(error) => warn!(%error, hash = %best, "Block audit skipped: candidate unavailable in published chain"),
                        }
                    } else { info!(hash = %best, "Block audit skipped: no fresh complete observation"); }
                }
                #[cfg(feature = "full-api")]
                {
                    warmed = Instant::now();
                }
                // After the publication, so no update waits for it. A walk that fails keeps the
                // last total.
                if let Err(error) = record_data_bytes(&query, &data_path, &logs_path) {
                    warn!("Disk usage not measured: {error}");
                }
                continue;
            }
        } else {
            updating_to = None;
            #[cfg(feature = "full-api")]
            { capture = None; }
            // Idle minutes leave what the next update reads on disk: read it again before it
            // goes cold, from the template the next block most likely shares.
            #[cfg(feature = "full-api")]
            if warmed.elapsed() >= WARM_INTERVAL {
                plugins.warm(&pool.load().template_transactions());
                warmed = Instant::now();
            }
        }
        sleep(Duration::from_secs(1));
    }
}

/// What the sync status and the disk usage report of the node, its tip height and the bytes of
/// its block files: read before the server answers and at each new tip, never per request.
fn record_node(client: &Client, query: &AsyncQuery) -> Result<()> {
    let (height, blocks_bytes) = client.get_tip_height_and_blocks_size()?;
    query.sync(|q| q.set_node_tip(height, blocks_bytes));
    Ok(())
}

/// What the disk usage reports of Bitview's own data: measured before the server answers and
/// after each publication, never per request.
fn record_data_bytes(query: &AsyncQuery, data_path: &Path, logs_path: &Path) -> Result<()> {
    let start = Instant::now();
    query.sync(|q| q.record_data_bytes(data_path, logs_path))?;
    debug!("Disk usage measured in {:.2?}", start.elapsed());
    Ok(())
}

/// A failed read keeps the list as it was.
#[cfg(feature = "full-api")]
fn refresh_stale_blocks(query: &bitview_query::Query) {
    if let Err(error) = query.refresh_stale_blocks() {
        warn!("Stale blocks not refreshed: {error}");
    }
}

fn server_stopped(runtime: &Runtime, handle: JoinHandle<Result<()>>) -> Result<()> {
    runtime.block_on(handle)??;
    Err(Error::Internal("HTTP server stopped unexpectedly"))
}
