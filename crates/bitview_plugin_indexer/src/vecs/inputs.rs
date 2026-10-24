use bitview_primitives::{TxInIndex, TxOutIndex, TypeIndex};
use bitview_traversable::{Compact, Traversable};
use brk_error::Result;
use brk_types::{Height, OutPoint, OutputType, Sats, TxIndex, Version};
use vecdb::{
    AnyStoredVec, Budgeted, Database, ImportableVec, PcoVec, Rw, StorageMode, UnaryTransform,
};

/// A coinbase input spends no output: it publishes 0, not the `Sats::MAX` it is stored as, and
/// none of its relations to one (null, not their maximum-value markers).
struct CoinbaseSpendsNothing;

impl UnaryTransform<OutPoint, Option<OutPoint>> for CoinbaseSpendsNothing {
    #[inline]
    fn apply(value: OutPoint) -> Option<OutPoint> {
        value.is_not_coinbase().then_some(value)
    }
}

impl UnaryTransform<TxOutIndex, Option<TxOutIndex>> for CoinbaseSpendsNothing {
    #[inline]
    fn apply(value: TxOutIndex) -> Option<TxOutIndex> {
        (value != TxOutIndex::COINBASE).then_some(value)
    }
}

impl UnaryTransform<TypeIndex, Option<TypeIndex>> for CoinbaseSpendsNothing {
    #[inline]
    fn apply(value: TypeIndex) -> Option<TypeIndex> {
        (value != TypeIndex::COINBASE).then_some(value)
    }
}

impl UnaryTransform<Sats, Sats> for CoinbaseSpendsNothing {
    #[inline]
    fn apply(value: Sats) -> Sats {
        if value.is_max() { Sats::ZERO } else { value }
    }
}

#[derive(Traversable)]
pub struct InputsVecs<M: StorageMode = Rw> {
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `height`, this is where the block begins and equals the number of
    /// inputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first input.
    pub first_txin_index: M::Stored<PcoVec<Height, TxInIndex, Budgeted>>,
    /// Previous-output reference encoded as the global transaction index and
    /// zero-based output position within that transaction; null for a coinbase
    /// input, which spends none.
    // Stored as the maximum of each component for a coinbase input.
    pub outpoint: Compact<PcoVec<TxInIndex, OutPoint>, Option<OutPoint>, M>,
    /// The output the input spends; null for a coinbase input.
    // Stored as `u64::MAX` for a coinbase input.
    pub txout_index: Compact<PcoVec<TxInIndex, TxOutIndex>, Option<TxOutIndex>, M>,
    /// The transaction containing the input.
    pub tx_index: M::Stored<PcoVec<TxInIndex, TxIndex>>,
    /// BRK locking-script classification of an output. At `txout_index`, this
    /// classifies the indexed output; at `txin_index`, it classifies the
    /// previous output spent by the input. Coinbase inputs use `unknown_script`.
    pub output_type: M::Stored<PcoVec<TxInIndex, OutputType>>,
    /// Zero-based index within the output's BRK type-specific collection. At
    /// `txout_index`, this identifies the indexed output; at `txin_index`, it
    /// identifies the previous output spent by the input, null for a coinbase
    /// input. Address types index distinct addresses, while other types index
    /// outputs in canonical order.
    // Stored as `u32::MAX` for a coinbase input.
    pub type_index: Compact<PcoVec<TxInIndex, TypeIndex>, Option<TypeIndex>, M>,
    /// Value of the output the input spends, in satoshis; 0 for a coinbase input, which
    /// spends none.
    // Stored as `Sats::MAX` for a coinbase input, which the stored vec's readers rely on.
    pub value: Compact<PcoVec<TxInIndex, Sats>, Sats, M>,
}

impl InputsVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (first_txin_index, outpoint, txout_index, tx_index, output_type, type_index, value) = parallel_import! {
            first_txin_index = PcoVec::import(db, "first_txin_index", version),
            outpoint = PcoVec::import(db, "outpoint", version),
            txout_index = PcoVec::import(db, "spent_txout_index", version),
            tx_index = PcoVec::import(db, "input_tx_index", version),
            output_type = PcoVec::import(db, "output_type", version),
            type_index = PcoVec::import(db, "type_index", version),
            value = PcoVec::import(db, "input_value", version),
        };
        Ok(Self {
            first_txin_index,
            outpoint: Compact::named_with::<CoinbaseSpendsNothing>(outpoint, "outpoint"),
            txout_index: Compact::named_with::<CoinbaseSpendsNothing>(
                txout_index,
                "spent_txout_index",
            ),
            tx_index,
            output_type,
            type_index: Compact::named_with::<CoinbaseSpendsNothing>(type_index, "type_index"),
            value: Compact::named_with::<CoinbaseSpendsNothing>(value, "input_value"),
        })
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_txin_index as &dyn AnyStoredVec,
            &*self.outpoint,
            &*self.txout_index,
            &self.tx_index,
            &self.output_type,
            &*self.type_index,
            &*self.value,
        ]
        .into_iter()
    }
}
