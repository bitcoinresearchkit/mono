use std::{
    env,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview::Computer;
use brk_alloc::Mimalloc;
use brk_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use vecdb::Exit;

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    brk_logger::init(Some(Path::new(".log")))?;

    let bitcoin_dir = Client::default_bitcoin_path();
    // let bitcoin_dir = Path::new("/Volumes/WD_BLACK/bitcoin");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    // let outputs_dir = Path::new("../../_outputs");

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);

    let mut indexer = Indexer::import(&outputs_dir, &reader)?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    // Pre-run indexer if too far behind, then drop and reimport to reduce memory
    let chain_height = client.get_last_height()?;
    let indexed_height = indexer.vecs().next_height();
    if u32::from(chain_height).saturating_sub(u32::from(indexed_height)) > 1000 {
        indexer.checked_index(&exit)?;
        drop(indexer);
        Mimalloc::collect();
        indexer = Indexer::import(&outputs_dir, &reader)?;
    }

    let mut computer = Computer::forced_import(&outputs_dir, &indexer)?;

    loop {
        let i = Instant::now();
        indexer.checked_index(&exit)?;

        Mimalloc::collect();

        computer.compute(&mut indexer, &exit)?;
        dbg!(i.elapsed());
        sleep(Duration::from_secs(10));
    }
}
