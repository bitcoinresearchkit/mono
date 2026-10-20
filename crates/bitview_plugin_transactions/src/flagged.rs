use bitview_primitives::{Boolean, Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    IndexSources, LazyPercentCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling,
    ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, TxIndex, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, ReadableCloneableVec, Rw, StorageMode};

/// A kind of transaction: its per-transaction flag beside its per-block count.
#[derive(Clone, Traversable)]
pub struct Flagged<F, C> {
    /// Whether the transaction is one of them.
    pub flag: F,
    #[traversable(flatten)]
    pub count: C,
}

/// A kind of transaction's per-block count (`<kind>_count`, as `p2tr_tx_count`), with its cumulative
/// and window sums, and its share of all transactions over the same spans (`<kind>_share_<window>`).
#[derive(Traversable, Deref, DerefMut)]
pub struct TxCount<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub count: PerBlockCumulativeRolling<Count, M>,
    /// Their share of all transactions, coinbase included.
    pub share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl TxCount {
    /// `kind` is the ids' prefix (`p2tr_tx`).
    pub(crate) fn import(
        db: &Database,
        kind: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
        all: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let count = PerBlockCumulativeRolling::import(
            db,
            &format!("{kind}_count"),
            version,
            indexes,
            window_starts,
        )?;
        let share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Count,
            Count,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{kind}_share"),
            version,
            count.cumulative_source(),
            all,
            window_starts,
            indexes,
        );
        Ok(Self { count, share })
    }
}

/// A kind of transaction this plugin classifies: its stored flag beside its windowed count.
pub(crate) type Classified<M> =
    Flagged<<M as StorageMode>::Stored<PcoVec<TxIndex, Boolean>>, TxCount<M>>;

impl Classified<Rw> {
    /// `kind` is the count's id prefix (`coinjoin_tx`).
    pub(crate) fn import(
        db: &Database,
        flag: &str,
        kind: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
        all: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        Ok(Self {
            flag: PcoVec::import(db, flag, version)?,
            count: TxCount::import(db, kind, version, indexes, window_starts, all)?,
        })
    }

    /// The stored flag and count: per transaction, then per block.
    pub(crate) fn stored_mut(&mut self) -> (&mut dyn AnyStoredVec, &mut dyn AnyStoredVec) {
        (&mut self.flag, self.count.stored_mut())
    }
}
