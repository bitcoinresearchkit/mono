use brk_traversable::Traversable;
use brk_types::StoredU64;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::CpfpRoleId;
use crate::internal::{ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling};

#[derive(Deref, DerefMut, Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Number of confirmed transactions whose same-block descendants raise
    /// their fee rate under Single Fee Linearization.
    pub cpfp_parent: LazyColumnPerBlockCumulativeRolling<StoredU64, CpfpRoleId>,
    /// Number of confirmed transactions whose fee raises the effective rate of
    /// a same-block ancestor-closed chunk under Single Fee Linearization.
    pub cpfp_child: LazyColumnPerBlockCumulativeRolling<StoredU64, CpfpRoleId>,
    #[deref]
    #[deref_mut]
    #[traversable(hidden)]
    pub source: ColumnarPerBlockCumulativeRolling<StoredU64, CpfpRoleId, (), M>,
}
