use std::{
    env, fs,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview::{DefaultPlugins, bootstrap, update};
use bitview_bencher::Bencher;
use bitview_plugin_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tracing::{debug, info};
use vecdb::Exit;

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let bitcoin_dir = Client::default_bitcoin_path();
    // let bitcoin_dir = Path::new("/Volumes/WD_BLACK1/bitcoin");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    // let outputs_dir = Path::new("/Volumes/WD_BLACK1/bitview");
    fs::create_dir_all(&outputs_dir)?;

    brk_logger::init(Some(&outputs_dir.join("logs")))?;

    let mut bencher = Bencher::from_cargo_env("bitview", &outputs_dir)?;
    bencher.start()?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();
    let bencher_clone = bencher.clone();
    exit.register_cleanup(move || {
        let _ = bencher_clone.stop();
        debug!("Bench stopped.");
    });

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);

    let mut plugins = bootstrap(
        &outputs_dir,
        || {
            let indexer = Indexer::import(&outputs_dir, &reader)?;
            DefaultPlugins::forced_import(&outputs_dir, indexer)
        },
        &exit,
    )?;

    loop {
        let i = Instant::now();
        update(&mut plugins, &exit)?;
        info!("Done in {:?}", i.elapsed());

        sleep(Duration::from_secs(60));
    }
}
