use bitview_primitives::{Bytes32, Index40, TxInIndex, TxOutIndex, TxVersion};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{BlkPosition, Height, RawLockTime, SigOps, TxIndex, Txid, Version, Weight};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, Budgeted, BytesVec, Database, ImportableVec, LazyVec, PcoVec,
    ReadableCloneableVec, Rw, Stamp, StorageMode, WritableVec,
};

pub mod features;
pub mod metadata;
pub mod versions;

pub use features::TransactionFeaturesVecs;
pub use features::{BlockCount, FeatureVecs, FlagView, TransactionCounts, TxFeatureFlags};
pub use metadata::TxMetadataVecs;
pub use versions::VersionCountVecs;

#[derive(Traversable)]
pub struct TransactionsVecs<M: StorageMode = Rw> {
    /// Global zero-based transaction index at which the indexed block begins,
    /// equal to the number of transactions in all preceding blocks.
    pub first_tx_index: M::Stored<PcoVec<Height, TxIndex, Budgeted>>,
    /// Transaction ID: the double-SHA256 hash of the transaction's non-witness
    /// serialization, displayed in Bitcoin's conventional hexadecimal byte
    /// order.
    pub txid: M::Stored<BytesVec<TxIndex, Txid>>,
    /// Compact transaction-version category for the indexed transaction. Values
    /// 1, 2, and 3 preserve those exact signed 32-bit Bitcoin transaction
    /// versions; 255 represents every other version. The series includes
    /// coinbase transactions. Use individual raw transaction data to inspect
    /// the original version when this value is 255.
    #[traversable(rename = "version")]
    pub tx_version: M::Stored<PcoVec<TxIndex, TxVersion>>,
    /// Raw transaction `nLockTime`. Values below 500,000,000 represent block
    /// heights and values at or above it represent Unix timestamps; zero
    /// disables absolute locktime. This does not account for whether input
    /// sequence numbers make the locktime effective.
    #[traversable(rename = "locktime")]
    pub raw_locktime: M::Stored<PcoVec<TxIndex, RawLockTime>>,
    /// BIP-141 transaction weight in weight units: non-witness bytes count as
    /// four weight units and witness bytes count as one.
    pub weight: M::Stored<PcoVec<TxIndex, Weight>>,
    /// Serialized size in bytes, including witness data: the byte length of the
    /// transaction's consensus serialization.
    #[traversable(rename = "size")]
    pub total_size: M::Stored<PcoVec<TxIndex, Bytes32>>,
    /// BIP-141 signature-operation cost: legacy scriptPubKey, scriptSig, and
    /// P2SH redeem-script sigops cost four units; P2WPKH and P2WSH sigops cost
    /// one. This is a static count, not the number of signatures executed.
    /// Tapscript sigops are excluded because BIP-342 uses a separate per-input
    /// budget.
    #[traversable(rename = "sigop_cost")]
    pub total_sigop_cost: M::Stored<PcoVec<TxIndex, SigOps>>,
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `height`, this is where the block begins and equals the number of
    /// inputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first input.
    pub first_txin_index: M::Stored<PcoVec<TxIndex, TxInIndex>>,
    /// Stored in 5 bytes per transaction; series readers see `first_txout_index_view`.
    #[traversable(hidden)]
    pub first_txout_index: M::Stored<BytesVec<TxIndex, Index40<TxOutIndex>>>,
    /// Global zero-based transaction-output index in canonical blockchain
    /// order. At `height`, this is where the block begins and equals the number
    /// of outputs in preceding blocks; at `tx_index`, it identifies the
    /// transaction's first output.
    #[traversable(rename = "first_txout_index")]
    pub first_txout_index_view: LazyVec<TxIndex, TxOutIndex, TxIndex, Index40<TxOutIndex>>,
    pub features: TransactionFeaturesVecs<M>,
    /// Counts every transaction, including coinbase, by its signed 32-bit
    /// Bitcoin transaction version.
    pub versions: VersionCountVecs<M>,
    #[traversable(hidden)]
    pub position: M::Stored<PcoVec<TxIndex, BlkPosition>>,
}

impl TransactionsVecs {
    pub fn split_for_finalize(
        &mut self,
    ) -> (
        &mut BytesVec<TxIndex, Index40<TxOutIndex>>,
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
                features: &mut self.features,
                versions: &mut self.versions,
            },
        )
    }

    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (
            first_tx_index,
            txid,
            tx_version,
            raw_locktime,
            weight,
            total_size,
            total_sigop_cost,
            first_txin_index,
            first_txout_index,
            features,
            versions,
            position,
        ) = parallel_import! {
            first_tx_index = PcoVec::import(db, "first_tx_index", version),
            txid = BytesVec::import(db, "txid", version),
            tx_version = PcoVec::import(db, "tx_version", version),
            raw_locktime = PcoVec::import(db, "tx_locktime", version),
            weight = PcoVec::import(db, "tx_weight", version),
            total_size = PcoVec::import(db, "tx_size", version),
            total_sigop_cost = PcoVec::import(db, "tx_sigop_cost", version),
            first_txin_index = PcoVec::import(db, "first_txin_index", version),
            first_txout_index = BytesVec::import(db, "first_txout_index", version),
            features = TransactionFeaturesVecs::import(db, version),
            versions = VersionCountVecs::import(db, version),
            position = PcoVec::import(db, "tx_position", version),
        };
        Ok(Self {
            first_tx_index,
            txid,
            tx_version,
            raw_locktime,
            weight,
            total_size,
            total_sigop_cost,
            first_txin_index,
            first_txout_index_view: LazyVec::init(
                "first_txout_index",
                Version::ZERO,
                first_txout_index.read_only_boxed_clone(),
                |_, index| index.get(),
            ),
            first_txout_index,
            features,
            versions,
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
        self.first_txin_index
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.first_txout_index
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.position
            .truncate_if_needed_with_stamp(tx_index, stamp)?;
        self.features.truncate(height, tx_index, stamp)?;
        self.versions.truncate(height, stamp)
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
            &mut self.first_txin_index,
            &mut self.first_txout_index,
            &mut self.position,
        ]
        .into_par_iter()
        .chain(self.features.par_iter_mut_any())
        .chain(self.versions.par_iter_mut_any())
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
            &self.first_txin_index,
            &self.first_txout_index,
            &self.position,
        ]
        .into_iter()
        .chain(self.features.iter_any())
        .chain(self.versions.iter_any())
    }
}
