use bitview_primitives::{Boolean, Count};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::{TxIndex, Version};
use vecdb::{AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, Rw, StorageMode};

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

impl Classified<Rw> {
    pub(crate) fn import(
        db: &Database,
        flag: &str,
        count: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            flag: EagerVec::import(db, flag, version)?,
            count: PerBlockCumulativeRolling::import(db, count, version, indexes, window_starts)?,
        })
    }

    /// The stored flag and count: per transaction, then per block.
    pub(crate) fn stored_mut(&mut self) -> (&mut dyn AnyStoredVec, &mut dyn AnyStoredVec) {
        (&mut self.flag, self.count.stored_mut())
    }
}
