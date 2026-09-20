use std::{env, fs, path::Path, thread};

use bitview::ImportContext;
use bitview_default::DefaultPlugins;
use bitview_query::Query;
use brk_error::Result;
use brk_exit::Exit;
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Addr;
use vecdb::Budgeted;

pub fn main() -> Result<()> {
    Budgeted::init_global(2 * 1024 * 1024 * 1024)?;
    let bitcoin_dir = Client::default_bitcoin_path();

    let blocks_dir = bitcoin_dir.join("blocks");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    fs::create_dir_all(&outputs_dir)?;

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let reader = Reader::new(blocks_dir, &client);
    let context = ImportContext::new(&outputs_dir);

    let plugins = DefaultPlugins::import(context, &reader)?;

    let mut mempool = Mempool::new(&client);
    let read_only = mempool.read_only_clone();
    thread::spawn(move || {
        mempool.start();
    });

    let query = Query::build(&plugins, Some(read_only));

    let _ = dbg!(query.addr(Addr::from(
        "bc1qwzrryqr3ja8w7hnja2spmkgfdcgvqwp5swz4af4ngsjecfz0w0pqud7k38".to_string(),
    )));

    let _ = dbg!(query.addr_txids(
        Addr::from("bc1qwzrryqr3ja8w7hnja2spmkgfdcgvqwp5swz4af4ngsjecfz0w0pqud7k38".to_string()),
        None,
        25
    ));

    let _ = dbg!(query.addr_utxos(
        Addr::from("bc1qwzrryqr3ja8w7hnja2spmkgfdcgvqwp5swz4af4ngsjecfz0w0pqud7k38".to_string()),
        1000,
    ));

    Ok(())
}
