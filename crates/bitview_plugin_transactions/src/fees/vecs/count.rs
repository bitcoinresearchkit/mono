use bitview_primitives::StoredU64;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Number of confirmed transactions whose same-block descendants raise
    /// their fee rate under Single Fee Linearization.
    pub cpfp_parent: PerBlockCumulativeRolling<StoredU64, M>,
    /// Number of confirmed transactions whose fee raises the effective rate of
    /// a same-block ancestor-closed chunk under Single Fee Linearization.
    pub cpfp_child: PerBlockCumulativeRolling<StoredU64, M>,
}

impl CountVecs {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<StoredU64>> {
        [&mut self.cpfp_parent, &mut self.cpfp_child].into_iter()
    }
}
