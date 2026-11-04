use bitview_primitives::{Boolean, Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    IndexSources, LazyPercentCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling,
    ValuePerBlockCumulativeRolling, ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, TxIndex, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, Database, ImportableVec, PcoVec, ReadableCloneableVec, ReadableVec, Rw,
    StorageMode,
};

/// A kind of transaction: its per-transaction flag beside its per-block count.
#[derive(Clone, Traversable)]
pub struct Flagged<F, C> {
    /// Whether the transaction is one of them.
    pub flag: F,
    #[traversable(flatten)]
    pub count: C,
}

/// The wholes a kind of transaction's shares divide: all transactions, coinbase included, and
/// transfer volume, each through every block.
pub(crate) struct Totals<'a, T, V> {
    pub txs: &'a T,
    pub volume: &'a V,
}

// (By hand: derived, they'd ask the sources to be `Copy` too.)
impl<T, V> Clone for Totals<'_, T, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, V> Copy for Totals<'_, T, V> {}

/// A kind of transaction's per-block count (`<kind>_count`, as `p2tr_tx_count`), with its cumulative
/// and window sums, and its share of all transactions over the same spans (`<kind>_share_<window>`);
/// and the bitcoin they moved (`<kind>_value`), likewise.
#[derive(Traversable, Deref, DerefMut)]
pub struct TxCount<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub count: PerBlockCumulativeRolling<Count, M>,
    /// Their share of all transactions, coinbase included.
    pub share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    pub value: TxValue<M>,
}

/// The bitcoin a kind of transaction moved: the value of their inputs, the coinbase's none, as
/// transfer volume counts it; with its cumulative and window sums, and its share of transfer volume
/// over the same spans (`<kind>_value_share_<window>`).
#[derive(Traversable, Deref, DerefMut)]
pub struct TxValue<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub value: ValuePerBlockCumulativeRolling<M>,
    /// Their share of transfer volume.
    pub share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl TxValue {
    /// A block's value, in sats: its dollars compute apart (`compute_cents`).
    #[inline]
    pub(crate) fn push(&mut self, value: Sats) {
        self.value.push_sats(value);
    }

    /// The stored sats, per block (the dollars resume by them).
    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.value.cumulative.sats.height
    }

    /// Each block's dollars, its sats at its price, from `start`.
    pub(crate) fn compute_cents(
        &mut self,
        start: Height,
        price: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.value.compute_cents(start, price, exit)
    }
}

impl TxCount {
    /// `kind` is the ids' prefix (`p2tr_tx`).
    pub(crate) fn import(
        db: &Database,
        kind: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
        totals: Totals<
            '_,
            impl ReadableCloneableVec<Height, Count>,
            impl ReadableCloneableVec<Height, Sats>,
        >,
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
            totals.txs,
            window_starts,
            indexes,
        );
        let value = ValuePerBlockCumulativeRolling::import(
            db,
            &format!("{kind}_value"),
            version,
            indexes,
            window_starts,
        )?;
        let value_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{kind}_value_share"),
            version,
            value.cumulative.sats.resolutions.height_source(),
            totals.volume,
            window_starts,
            indexes,
        );
        Ok(Self {
            count,
            share,
            value: TxValue {
                value,
                share: value_share,
            },
        })
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
        totals: Totals<
            '_,
            impl ReadableCloneableVec<Height, Count>,
            impl ReadableCloneableVec<Height, Sats>,
        >,
    ) -> Result<Self> {
        Ok(Self {
            flag: PcoVec::import(db, flag, version)?,
            count: TxCount::import(db, kind, version, indexes, window_starts, totals)?,
        })
    }

    /// The stored flag, then count and value: per transaction, then per block.
    pub(crate) fn stored_mut(&mut self) -> (&mut dyn AnyStoredVec, [&mut dyn AnyStoredVec; 2]) {
        let TxCount { count, value, .. } = &mut self.count;
        (&mut self.flag, [count.stored_mut(), value.stored_mut()])
    }
}
