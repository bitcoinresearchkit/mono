use std::path::Path;

use bitview_query::AsyncQuery;
use bitview_runtime::Computer;
use bitview_server::{Server, ServerConfig, Website};
use brk_error::Result;
use brk_indexer::Indexer;
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tracing::info;
use vecdb::Exit;

pub fn main() -> Result<()> {
    brk_logger::init(Some(Path::new(".log")))?;

    let bitcoin_dir = Client::default_bitcoin_path();
    let outputs_dir = Path::new(&std::env::var("HOME").unwrap()).join(".bitview");

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    let indexer = Indexer::import(&outputs_dir, &reader)?;
    let computer = Computer::forced_import(&outputs_dir, &indexer)?;

    let mempool = Mempool::new(&client);
    let mempool_clone = mempool.clone();
    std::thread::spawn(move || {
        mempool_clone.start();
    });

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let query = AsyncQuery::build(&indexer, &computer, Some(mempool));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Option 1: block_on to run and properly propagate errors
    runtime.block_on(async move {
        let server = Server::new(
            &query,
            ServerConfig {
                data_path: outputs_dir,
                website: Website::Disabled,
                ..Default::default()
            },
        );

        let handle = tokio::spawn(async move { server.serve(None).await });

        // Await the handle to catch both panics and errors
        match handle.await {
            Ok(Ok(())) => info!("Server shut down cleanly"),
            Ok(Err(e)) => tracing::error!("Server error: {e:?}"),
            Err(e) => {
                // JoinError - either panic or cancellation
                if e.is_panic() {
                    tracing::error!("Server panicked: {:?}", e.into_panic());
                } else {
                    tracing::error!("Server task cancelled");
                }
            }
        }

        Ok(()) as Result<()>
    })
}
