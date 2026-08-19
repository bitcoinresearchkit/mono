use std::{fs, path::Path};

use bitview_plugin_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use vecdb::ReadableVec;

fn main() -> brk_error::Result<()> {
    brk_logger::init(Some(Path::new(".log")))?;

    let outputs_dir = Path::new(&std::env::var("HOME").unwrap()).join(".bitview");
    fs::create_dir_all(&outputs_dir)?;

    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;
    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    let indexer = Indexer::import(&outputs_dir, &reader)?;

    println!(
        "{:?}",
        indexer.vecs().outputs.value.collect_range_at(0, 200)
    );

    Ok(())
}
