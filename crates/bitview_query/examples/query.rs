use std::{env, fs, path::Path};

use bitview_composition::DefaultPlugins;
use bitview_plugin_indexer::Indexer;
use bitview_query::Query;
use brk_error::Result;
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Addr;
use vecdb::Exit;

pub fn main() -> Result<()> {
    let bitcoin_dir = Client::default_bitcoin_path();
    // let bitcoin_dir = Path::new("/Volumes/WD_BLACK1/bitcoin");

    let blocks_dir = bitcoin_dir.join("blocks");

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");
    fs::create_dir_all(&outputs_dir)?;
    // let outputs_dir = Path::new("/Volumes/WD_BLACK1/bitview");

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let reader = Reader::new(blocks_dir, &client);

    let indexer = Indexer::import(&outputs_dir, &reader)?;

    let plugins = DefaultPlugins::forced_import(&outputs_dir, indexer)?;

    let mempool = Mempool::new(&client);
    let mempool_clone = mempool.clone();
    std::thread::spawn(move || {
        mempool_clone.start();
    });

    let query = Query::build(&plugins, Some(mempool));

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

    // dbg!(query.search_and_format(SeriesSelection {
    //     index: Index::Height,
    //     series: vec!["date"].into(),
    //     range: DataRangeFormat::default().set_from(-1),
    // })?);
    // dbg!(query.search_and_format(SeriesSelection {
    //     index: Index::Height,
    //     series: vec!["date", "timestamp"].into(),
    //     range: DataRangeFormat::default().set_from(-10).set_count(5),
    // })?);

    Ok(())
}
