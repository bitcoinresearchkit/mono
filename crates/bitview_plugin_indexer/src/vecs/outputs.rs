use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, OutputType, Sats, TxOutIndex, TypeIndex, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, OverflowVec, PcoVec, Rw, Stamp, StorageMode,
    WritableVec,
};

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    /// Global zero-based transaction-output index in canonical blockchain
    /// order. At `height`, this is where the block begins and equals the number
    /// of outputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first output.
    pub first_txout_index: M::Stored<PcoVec<Height, TxOutIndex>>,
    /// Value in satoshis of the indexed transaction output. At `txout_index`,
    /// this is the output's value; at `txin_index`, it is the value of the
    /// previous output spent by the input. Coinbase inputs use `Sats::MAX`
    /// because they have no previous output.
    pub value: M::Stored<OverflowVec<TxOutIndex, Sats>>,
    /// BRK locking-script classification of an output. At `txout_index`, this
    /// classifies the indexed output; at `txin_index`, it classifies the
    /// previous output spent by the input. Coinbase inputs use `unknown`.
    pub output_type: M::Stored<BytesVec<TxOutIndex, OutputType>>,
    /// Zero-based index within the output's BRK type-specific collection. At
    /// `txout_index`, this identifies the indexed output; at `txin_index`, it
    /// identifies the previous output spent by the input. Address types index
    /// distinct addresses, while other types index outputs in canonical order.
    /// Coinbase inputs use `u32::MAX`.
    pub type_index: M::Stored<BytesVec<TxOutIndex, TypeIndex>>,
}

impl OutputsVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (first_txout_index, value, output_type, type_index) = parallel_import! {
            first_txout_index = PcoVec::forced_import(db, "first_txout_index", version),
            value = OverflowVec::forced_import(db, "value", version),
            output_type = BytesVec::forced_import(db, "output_type", version),
            type_index = BytesVec::forced_import(db, "type_index", version),
        };
        Ok(Self {
            first_txout_index,
            value,
            output_type,
            type_index,
        })
    }

    pub fn truncate(
        &mut self,
        height: Height,
        txout_index: TxOutIndex,
        stamp: Stamp,
    ) -> Result<()> {
        self.first_txout_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.value
            .truncate_if_needed_with_stamp(txout_index, stamp)?;
        self.output_type
            .truncate_if_needed_with_stamp(txout_index, stamp)?;
        self.type_index
            .truncate_if_needed_with_stamp(txout_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.first_txout_index as &mut dyn AnyStoredVec,
            &mut self.value,
            &mut self.output_type,
            &mut self.type_index,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_txout_index as &dyn AnyStoredVec,
            &self.value,
            &self.output_type,
            &self.type_index,
        ]
        .into_iter()
    }
}
