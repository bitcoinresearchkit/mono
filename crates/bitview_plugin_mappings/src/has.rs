use vecdb::StorageMode;

use crate::Vecs;

pub trait HasMappings<M: StorageMode> {
    fn mappings(&self) -> &Vecs<M>;
}
