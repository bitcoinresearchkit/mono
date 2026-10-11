use bitview_primitives::{Boolean, Count};
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::TxIndex;
use vecdb::{EagerVec, PcoVec, StorageMode};

/// A kind of transaction: its per-transaction flag beside its per-block count.
#[derive(Clone, Traversable)]
pub struct Flagged<F, C> {
    /// Whether the transaction is one of them.
    pub flag: F,
    #[traversable(flatten)]
    pub count: C,
}

/// A per-block count without windows.
#[derive(Clone, Traversable)]
pub struct Block<V> {
    /// Value for the represented block.
    pub block: V,
}

/// A kind of transaction this plugin classifies: its stored flag beside its windowed count.
pub(crate) type Classified<M> = Flagged<
    <M as StorageMode>::Stored<EagerVec<PcoVec<TxIndex, Boolean>>>,
    PerBlockCumulativeRolling<Count, M>,
>;
