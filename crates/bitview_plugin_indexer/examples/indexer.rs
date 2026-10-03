use std::{
    env, fs,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use brk_alloc::Mimalloc;
use brk_exit::Exit;
use brk_logger::init;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use color_eyre::{Result, install};
use tracing::{debug, info};
use vecdb::Budgeted;

fn main() -> Result<()> {
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET)?;
    install()?;

    init(Some(Path::new(".log")))?;

    let node = ConnectArgs::default();

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    fs::create_dir_all(&outputs_dir)?;

    let client = node.client()?;

    let reader = Reader::new(node.blocks_dir(), &client);
    debug!("Reader created.");

    let context = ImportContext::new(&outputs_dir);
    let mut indexer = Indexer::import(context, &reader)?;
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
