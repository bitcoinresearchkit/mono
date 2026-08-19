use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the market plugin.
pub trait HasMarket<M: StorageMode> {
    fn market(&self) -> &Vecs<M>;
}
