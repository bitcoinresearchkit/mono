use brk_error::Result;

use std::{env, path::Path, time::Instant};

use bitview::Computer;
use bitview_bencher::Bencher;
use brk_alloc::Mimalloc;
use brk_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tracing::{debug, info};
use vecdb::Exit;

pub fn main() -> Result<()> {
    brk_logger::init(None)?;

    let bitcoin_dir = Client::default_bitcoin_path();
    // let bitcoin_dir = Path::new("/Volumes/WD_BLACK/bitcoin");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    let outputs_benches_dir = outputs_dir.join("benches");
    // let outputs_dir = Path::new("../../_outputs");

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);

    let mut indexer = Indexer::import(&outputs_dir, &reader)?;

    let mut computer = Computer::forced_import(&outputs_benches_dir, &indexer)?;

    let mut bencher =
        Bencher::from_cargo_env(env!("CARGO_PKG_NAME"), &outputs_dir.join("computed"))?;
    bencher.start()?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();
    let bencher_clone = bencher.clone();
    exit.register_cleanup(move || {
        let _ = bencher_clone.stop();
        debug!("Bench stopped.");
    });

    let i = Instant::now();
    indexer.index(&exit)?;
    info!("Done in {:?}", i.elapsed());

    Mimalloc::collect();

    let i = Instant::now();
    computer.compute(&mut indexer, &exit)?;
    info!("Done in {:?}", i.elapsed());

    // We want to benchmark the drop too
    drop(computer);
    drop(indexer);

    Ok(())
}
