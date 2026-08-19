#![doc = include_str!("../README.md")]

use brk_error::Result;

use std::{
    thread::{self, sleep},
    time::{Duration, Instant},
};

use bitview_plugin::ComputePlugin;
use bitview_query::AsyncQuery;
pub use bitview_runtime::Computer;
use bitview_server::{Server, ServerConfig};
use brk_alloc::Mimalloc;
use brk_indexer::Indexer;
use brk_mempool::Mempool;
use brk_reader::Reader;
use tracing::info;
use vecdb::Exit;

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
    let config = Config::import()?;

    brk_logger::init(Some(&default_logs_dir()))?;

    let client = config.rpc()?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let reader = Reader::new(config.blocksdir(), &client);

    let mut indexer = Indexer::import(&config.bitviewdir(), &reader)?;

    #[cfg(not(debug_assertions))]
    {
        // Pre-run indexer if too far behind, then drop and reimport to reduce memory
        let chain_height = client.get_last_height()?;
        let indexed_height = indexer.vecs().next_height();
        let blocks_behind = chain_height.saturating_sub(*indexed_height);
        if blocks_behind > 10_000 {
            info!("---");
            info!("Indexing {blocks_behind} blocks before starting server...");
            info!("---");
            sleep(Duration::from_secs(10));
            indexer.compute((), &exit)?;
            drop(indexer);
            Mimalloc::collect();
            indexer = Indexer::import(&config.bitviewdir(), &reader)?;
        }
    }

    let mut computer = Computer::forced_import(&config.bitviewdir(), &indexer)?;

    let mempool = Mempool::new(&client);

    indexer.begin_update();
    let query = AsyncQuery::build(&indexer, &computer, Some(mempool.clone()));

    let mempool_clone = mempool.clone();
    let resolver = query.sync(|q| q.indexer_prevout_resolver());
    thread::spawn(move || {
        mempool_clone.start_with(resolver);
    });

    let server_config = ServerConfig {
        data_path: config.bitviewdir(),
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

    loop {
        client.wait_for_synced_node()?;

        let last_height = client.get_last_height()?;

        info!("{} blocks found.", u32::from(last_height) + 1);

        let total_start = Instant::now();

        indexer.compute((), &exit)?;

        Mimalloc::collect();

        computer.compute(&mut indexer, &exit)?;

        info!("Total time: {:?}", total_start.elapsed());
        info!("Waiting for new blocks...");

        while last_height == client.get_last_height()? {
            sleep(Duration::from_secs(1))
        }
    }
}
