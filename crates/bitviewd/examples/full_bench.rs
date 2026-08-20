use std::{
    env, fs,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview::{ImportContext, UpdateContext, bootstrap, update};
use bitview_bencher::Bencher;
use bitview_default::DefaultPlugins;
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
    let import_context = ImportContext::new(&outputs_dir);
    let update_context = UpdateContext::new(&exit);
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
        import_context,
        |context| DefaultPlugins::import(context, &reader),
        update_context,
    )?;

    loop {
        let i = Instant::now();
        update(&mut plugins, update_context)?;
        info!("Done in {:?}", i.elapsed());

        sleep(Duration::from_secs(60));
    }
}
