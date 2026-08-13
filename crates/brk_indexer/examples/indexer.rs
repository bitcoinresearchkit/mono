use std::{
    env, fs,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use brk_alloc::Mimalloc;
use brk_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tracing::{debug, info};
use vecdb::Exit;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    brk_logger::init(Some(Path::new(".log")))?;

    let bitcoin_dir = Client::default_bitcoin_path();
    // let bitcoin_dir = Path::new("/Volumes/WD_BLACK1/bitcoin");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".brk");
    fs::create_dir_all(&outputs_dir)?;
    // let outputs_dir = Path::new("/Volumes/WD_BLACK1/brk");

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    debug!("Reader created.");

    let mut indexer = Indexer::import(&outputs_dir, &reader)?;
    debug!("Indexer imported.");

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    loop {
        let i = Instant::now();
        indexer.checked_index(&exit)?;
        indexer.finish_update()?;
        info!("Done in {:?}", i.elapsed());

        Mimalloc::collect();

        sleep(Duration::from_secs(60));
    }
}
