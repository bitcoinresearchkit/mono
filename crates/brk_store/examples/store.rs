use std::path::Path;

use brk_store::{Kind, Store, open_database};
use brk_types::{Height, TxIndex, Version};

fn main() -> brk_error::Result<()> {
    let path = Path::new("./examples/_fjall");
    let db = open_database(path)?;
    let mut store: Store<TxIndex, Height> =
        Store::import(&db, path, "numbers", Version::ZERO, Kind::Random)?;

    let key = TxIndex::new(10);
    let value = Height::new(50);
    store.insert(key, value);

    if let Some(ingest) = store.take_pending_ingest() {
        ingest.run()?;
    }
    assert_eq!(store.get(&key)?.as_deref(), Some(&value));

    Ok(())
}
