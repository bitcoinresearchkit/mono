use vecdb::StorageMode;

use crate::Indexer;

pub trait HasIndexer<M: StorageMode> {
    fn indexer(&self) -> &Indexer<M>;
}
