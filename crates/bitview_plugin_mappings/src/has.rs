use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the mappings plugin.
pub trait HasMappings<M: StorageMode> {
    fn mappings(&self) -> &Vecs<M>;
}
