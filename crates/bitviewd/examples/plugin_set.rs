use std::{
    env,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview::{ImportContext, UpdateContext, bootstrap, update};
use bitview_default::DefaultPlugins;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};

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

    let exit = Exit::new();
    exit.set_ctrlc_handler();
    let import_context = ImportContext::new(&outputs_dir);
    let update_context = UpdateContext::new(&exit);

    let mut plugins = bootstrap(
        import_context,
        |context| DefaultPlugins::import(context, &reader),
        update_context,
    )?;

    loop {
        let i = Instant::now();
        update(&mut plugins, update_context)?;
        dbg!(i.elapsed());
        sleep(Duration::from_secs(10));
    }
}
