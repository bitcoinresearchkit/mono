use bitview_primitives::{TxInIndex, TxOutIndex, TypeIndex};
use bitview_traversable::{Compact, Traversable};
use brk_error::Result;
use brk_types::{Height, OutPoint, OutputType, Sats, TxIndex, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, Budgeted, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode,
    UnaryTransform, WritableVec,
};

/// A coinbase input spends no output: it publishes 0, not the `Sats::MAX` it is stored as.
struct CoinbaseSpendsNothing;

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
    /// zero-based output position within that transaction. Coinbase inputs use
    /// the maximum of each component (`tx_index: 4294967295, vout: 65535`).
    pub outpoint: M::Stored<PcoVec<TxInIndex, OutPoint>>,
    /// The output the input spends, `u64::MAX` for a coinbase input.
    pub txout_index: M::Stored<PcoVec<TxInIndex, TxOutIndex>>,
    /// The transaction containing the input.
    pub tx_index: M::Stored<PcoVec<TxInIndex, TxIndex>>,
    /// BRK locking-script classification of an output. At `txout_index`, this
    /// classifies the indexed output; at `txin_index`, it classifies the
    /// previous output spent by the input. Coinbase inputs use `unknown`.
    pub output_type: M::Stored<PcoVec<TxInIndex, OutputType>>,
    /// Zero-based index within the output's BRK type-specific collection. At
    /// `txout_index`, this identifies the indexed output; at `txin_index`, it
    /// identifies the previous output spent by the input. Address types index
    /// distinct addresses, while other types index outputs in canonical order.
    /// Coinbase inputs use `u32::MAX`.
    pub type_index: M::Stored<PcoVec<TxInIndex, TypeIndex>>,
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
            outpoint,
            txout_index,
            tx_index,
            output_type,
            type_index,
            value: Compact::named_with::<CoinbaseSpendsNothing>(value, "input_value"),
        })
    }

    pub fn truncate(&mut self, height: Height, txin_index: TxInIndex, stamp: Stamp) -> Result<()> {
        self.first_txin_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.outpoint
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.txout_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.tx_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.output_type
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.type_index
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        self.value
            .truncate_if_needed_with_stamp(txin_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.first_txin_index as &mut dyn AnyStoredVec,
            &mut self.outpoint,
            &mut self.txout_index,
            &mut self.tx_index,
            &mut self.output_type,
            &mut self.type_index,
            &mut *self.value,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_txin_index as &dyn AnyStoredVec,
            &self.outpoint,
            &self.txout_index,
            &self.tx_index,
            &self.output_type,
            &self.type_index,
            &*self.value,
        ]
        .into_iter()
    }
}
