use vecdb::StorageMode;

use super::Vecs;

/// Provides typed access to the near-full-blocks plugin.
pub trait HasNearFullBlocks<M: StorageMode> {
    fn near_full_blocks(&self) -> &Vecs<M>;
}
