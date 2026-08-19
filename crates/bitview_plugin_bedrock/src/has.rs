use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the Bedrock plugin.
pub trait HasBedrock<M: StorageMode> {
    fn bedrock(&self) -> &Vecs<M>;
}
