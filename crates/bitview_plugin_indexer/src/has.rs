use vecdb::StorageMode;

use crate::Indexer;

/// Provides access to the indexer plugin.
pub trait HasIndexer<M: StorageMode> {
    fn indexer(&self) -> &Indexer<M>;
}
