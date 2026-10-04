use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Number of confirmed transactions whose same-block descendants raise
    /// their fee rate under Single Fee Linearization.
    pub cpfp_parent: PerBlockCumulativeRolling<Count, M>,
    /// Number of confirmed transactions whose fee raises the effective rate of
    /// a same-block ancestor-closed chunk under Single Fee Linearization.
    pub cpfp_child: PerBlockCumulativeRolling<Count, M>,
}

impl CountVecs {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<Count>> {
        [&mut self.cpfp_parent, &mut self.cpfp_child].into_iter()
    }
}
