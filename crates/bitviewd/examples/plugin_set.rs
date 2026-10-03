use std::{
    env,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use bitview::{ImportContext, UpdateContext, bootstrap, update};
use bitview_default::DefaultPlugins;
use brk_exit::Exit;
use brk_logger::init;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use color_eyre::{Result, install};
use vecdb::Budgeted;

pub fn main() -> Result<()> {
    Budgeted::init_global(bitview::DEFAULT_CACHE_BUDGET)?;
    install()?;

    init(Some(Path::new(".log")))?;

    let node = ConnectArgs::default();

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");

    let client = node.client()?;

    let reader = Reader::new(node.blocks_dir(), &client);

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
