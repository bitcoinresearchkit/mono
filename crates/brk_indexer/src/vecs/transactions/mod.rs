mod features;
mod metadata;

pub use features::{TransactionCountVecs, TransactionFeaturesVecs};
pub(crate) use features::{TransactionCounts, TxFeatureFlags};
pub use metadata::TxMetadataVecs;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    BlkPosition, Height, RawLockTime, SigOps, StoredBool, StoredU32, TxInIndex, TxIndex,
    TxOutIndex, TxVersion, Txid, Version, Weight,
};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec,
};

use crate::parallel_import;

#[derive(Traversable)]
pub struct TransactionsVecs<M: StorageMode = Rw> {
    /// Global zero-based transaction index at which the indexed block begins,
    /// equal to the number of transactions in all preceding blocks.
    pub first_tx_index: M::Stored<PcoVec<Height, TxIndex>>,
    /// Transaction ID: the double-SHA256 hash of the transaction's non-witness
    /// serialization, displayed in Bitcoin's conventional hexadecimal byte
    /// order.
    pub txid: M::Stored<BytesVec<TxIndex, Txid>>,
    /// Compact transaction-version category for the indexed transaction. Values
    /// 1, 2, and 3 preserve those exact signed 32-bit Bitcoin transaction
    /// versions; 255 represents every other version. The series includes
    /// coinbase transactions. Use individual raw transaction data to inspect
    /// the original version when this value is 255.
    pub tx_version: M::Stored<PcoVec<TxIndex, TxVersion>>,
    /// Raw transaction `nLockTime`. Values below 500,000,000 represent block
    /// heights and values at or above it represent Unix timestamps; zero
    /// disables absolute locktime. This does not account for whether input
    /// sequence numbers make the locktime effective.
    pub raw_locktime: M::Stored<PcoVec<TxIndex, RawLockTime>>,
    /// BIP-141 transaction weight in weight units: non-witness bytes count as
    /// four weight units and witness bytes count as one. The transaction-index
    /// series gives each transaction's value. Distribution series count every
    /// transaction equally and include coinbase, either in the represented
    /// block or the six-block window ending there; time-period indexes take the
    /// value from the period's final block.
    pub weight: M::Stored<PcoVec<TxIndex, Weight>>,
    /// Total serialized size in bytes, including witness data. At `tx_index`,
    /// this is the byte length of the transaction's consensus serialization. At
    /// `height`, this is the entire block: its 80-byte header, transaction-count
    /// CompactSize, and every serialized transaction.
    pub total_size: M::Stored<PcoVec<TxIndex, StoredU32>>,
    /// BIP-141 signature-operation cost. At `tx_index`, this is the cost of the
    /// indexed transaction. At `height`, this is the sum across every
    /// transaction in the block, including coinbase. Sigops in legacy
    /// scriptPubKeys, scriptSigs, and P2SH redeemScripts cost four units;
    /// P2WPKH and P2WSH sigops cost one. This statically counts
    /// signature-checking operations rather than signatures actually executed.
    /// Tapscript signature opcodes are excluded because BIP-342 uses a separate
    /// per-input execution budget. The post-SegWit consensus block limit is
    /// 80,000 cost units.
    pub total_sigop_cost: M::Stored<PcoVec<TxIndex, SigOps>>,
    /// Whether at least one input has a sequence number below `0xfffffffe`, the
    /// explicit opt-in RBF signal defined by BIP 125. This is a mechanical
    /// sequence signal: it does not prove the transaction was replaceable or
    /// replaced, does not include inherited signaling, and does not account for
    /// full-RBF policy. Coinbase transactions are evaluated by the same sequence
    /// rule.
    pub is_explicitly_rbf: M::Stored<PcoVec<TxIndex, StoredBool>>,
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `height`, this is where the block begins and equals the number of
    /// inputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first input.
    pub first_txin_index: M::Stored<PcoVec<TxIndex, TxInIndex>>,
    /// Global zero-based transaction-output index in canonical blockchain
    /// order. At `height`, this is where the block begins and equals the number
    /// of outputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first output.
    pub first_txout_index: M::Stored<BytesVec<TxIndex, TxOutIndex>>,
    #[traversable(hidden)]
    pub position: M::Stored<PcoVec<TxIndex, BlkPosition>>,
}

impl TransactionsVecs {
    pub fn split_for_finalize(
        &mut self,
    ) -> (
        &mut BytesVec<TxIndex, TxOutIndex>,
        &mut PcoVec<TxIndex, TxInIndex>,
        TxMetadataVecs<'_>,
    ) {
        (
            &mut self.first_txout_index,
            &mut self.first_txin_index,
            TxMetadataVecs {
                tx_version: &mut self.tx_version,
                txid: &mut self.txid,
                raw_locktime: &mut self.raw_locktime,
                weight: &mut self.weight,
                total_size: &mut self.total_size,
                total_sigop_cost: &mut self.total_sigop_cost,
                is_explicitly_rbf: &mut self.is_explicitly_rbf,
            },
        )
    }

    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (
            first_tx_index,
            txid,
            tx_version,
            raw_locktime,
            weight,
            total_size,
            total_sigop_cost,
            is_explicitly_rbf,
            first_txin_index,
            first_txout_index,
            position,
        ) = parallel_import! {
            first_tx_index = PcoVec::forced_import(db, "first_tx_index", version),
            txid = BytesVec::forced_import(db, "txid", version),
            tx_version = PcoVec::forced_import(db, "tx_version", version),
            raw_locktime = PcoVec::forced_import(db, "raw_locktime", version),
            weight = PcoVec::forced_import(db, "tx_weight", version),
            total_size = PcoVec::forced_import(db, "total_size", version),
            total_sigop_cost = PcoVec::forced_import(db, "total_sigop_cost", version),
            is_explicitly_rbf = PcoVec::forced_import(db, "is_explicitly_rbf", version),
            first_txin_index = PcoVec::forced_import(db, "first_txin_index", version),
            first_txout_index = BytesVec::forced_import(db, "first_txout_index", version),
            position = PcoVec::forced_import(db, "tx_position", version),
        };
        Ok(Self {
            first_tx_index,
            txid,
            tx_version,
            raw_locktime,
            weight,
            total_size,
            total_sigop_cost,
            is_explicitly_rbf,
            first_txin_index,
            first_txout_index,
            position,
        })
    }

    pub fn truncate(&mut self, height: Height, tx_index: TxIndex, stamp: Stamp) -> Result<()> {
        self.first_tx_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.txid.truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.tx_version
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.raw_locktime
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.weight.truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.total_size
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.total_sigop_cost
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.is_explicitly_rbf
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.first_txin_index
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.first_txout_index
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.position
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.first_tx_index as &mut dyn AnyStoredVec,
            &mut self.txid,
            &mut self.tx_version,
            &mut self.raw_locktime,
            &mut self.weight,
            &mut self.total_size,
            &mut self.total_sigop_cost,
            &mut self.is_explicitly_rbf,
            &mut self.first_txin_index,
            &mut self.first_txout_index,
            &mut self.position,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_tx_index as &dyn AnyStoredVec,
            &self.txid,
            &self.tx_version,
            &self.raw_locktime,
            &self.weight,
            &self.total_size,
            &self.total_sigop_cost,
            &self.is_explicitly_rbf,
            &self.first_txin_index,
            &self.first_txout_index,
            &self.position,
        ]
        .into_iter()
    }
}
