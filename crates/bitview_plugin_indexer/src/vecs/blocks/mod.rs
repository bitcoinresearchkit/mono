use bitview_primitives::{Bytes32, Count16, Difficulty};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{BlkPosition, BlockHash, CoinbaseTag, Height, Timestamp, Version, Weight};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, Database, ImportableVec, PcoVec, Rw, Stamp,
    StorageMode, WritableVec,
};

pub mod median_time;

#[derive(Traversable)]
pub struct BlocksVecs<M: StorageMode = Rw> {
    /// Double-SHA256 hash of the block header, displayed in Bitcoin's
    /// conventional hexadecimal byte order.
    pub blockhash: M::Stored<BytesVec<Height, BlockHash>>,
    /// First 100 bytes of the coinbase transaction's first-input `scriptSig`,
    /// exposed as a string by mapping each byte to the same-valued Unicode code
    /// point. This is raw coinbase data, not a normalized mining-pool label.
    pub coinbase_tag: M::Stored<BytesVec<Height, CoinbaseTag>>,
    /// Mining difficulty encoded by the block header, calculated as Bitcoin's
    /// maximum target divided by this block's proof-of-work target.
    /// Public through the blocks plugin (`blocks.difficulty.block`, every index).
    #[traversable(hidden)]
    pub difficulty: M::Stored<PcoVec<Height, Difficulty, Budgeted>>,
    /// Unix timestamp in seconds associated with the indexed block or time
    /// period. Block-header timestamps are not guaranteed to increase between
    /// consecutive heights.
    pub timestamp: M::Stored<PcoVec<Height, Timestamp, Budgeted>>,
    /// Median of this block's timestamp and up to ten predecessors, choosing
    /// the upper middle for early even-length windows. Compressed on disk;
    /// response builders read bounded ranges without caching the full history.
    #[traversable(hidden)]
    pub median_time: M::Stored<PcoVec<Height, Timestamp>>,
    /// Serialized block size in bytes, including witness data: its 80-byte header,
    /// transaction-count CompactSize, and every serialized transaction.
    #[traversable(rename = "size")]
    pub total: M::Stored<PcoVec<Height, Bytes32>>,
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    pub weight: M::Stored<PcoVec<Height, Weight>>,
    #[traversable(hidden)]
    pub position: M::Stored<PcoVec<Height, BlkPosition>>,
    /// Number of non-coinbase transactions using SegWit serialization.
    #[traversable(rename = "segwit_tx_count")]
    pub segwit_txs: M::Stored<PcoVec<Height, Count16>>,
    /// Combined total serialized size in bytes of the block's non-coinbase
    /// SegWit transactions; excludes block overhead and all other transactions.
    pub segwit_size: M::Stored<PcoVec<Height, Bytes32>>,
    /// Combined BIP-141 weight in weight units of the block's non-coinbase
    /// SegWit transactions; excludes block overhead and all other transactions.
    pub segwit_weight: M::Stored<PcoVec<Height, Weight>>,
}

impl BlocksVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (
            blockhash,
            coinbase_tag,
            difficulty,
            timestamp,
            median_time,
            total,
            weight,
            position,
            segwit_txs,
            segwit_size,
            segwit_weight,
        ) = parallel_import! {
            blockhash = BytesVec::import(db, "blockhash", version),
            coinbase_tag = BytesVec::import(db, "coinbase_tag", version),
            difficulty = PcoVec::import(db, "difficulty", version),
            timestamp = PcoVec::import(db, "timestamp", version),
            median_time = PcoVec::import(db, "median_time", version),
            total_size = PcoVec::import(db, "block_size", version),
            weight = PcoVec::import(db, "block_weight", version),
            position = PcoVec::import(db, "block_position", version),
            segwit_txs = PcoVec::import(db, "segwit_tx_count", version),
            segwit_size = PcoVec::import(db, "segwit_size", version),
            segwit_weight = PcoVec::import(db, "segwit_weight", version),
        };
        let mut this = Self {
            blockhash,
            coinbase_tag,
            difficulty,
            timestamp,
            median_time,
            total,
            weight,
            position,
            segwit_txs,
            segwit_size,
            segwit_weight,
        };
        // Upgrade old databases from the authoritative timestamp column. A
        // mismatched checkpoint can contain another branch: rebuild, not append.
        if this.median_time.len() != this.timestamp.len()
            || this.median_time.stamp() != this.timestamp.stamp()
        {
            this.median_time.clear()?;
            this.compute_median_times()?;
            this.median_time.stamped_write(this.timestamp.stamp())?;
        }
        Ok(this)
    }

    pub fn truncate(&mut self, height: Height, stamp: Stamp) -> Result<()> {
        self.blockhash
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.coinbase_tag
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.difficulty
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.timestamp
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.median_time
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.total.truncate_if_needed_with_stamp(height, stamp)?;
        self.weight.truncate_if_needed_with_stamp(height, stamp)?;
        self.position.truncate_if_needed_with_stamp(height, stamp)?;
        self.segwit_txs
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.segwit_size
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.segwit_weight
            .truncate_if_needed_with_stamp(height, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.blockhash as &mut dyn AnyStoredVec,
            &mut self.coinbase_tag,
            &mut self.difficulty,
            &mut self.timestamp,
            &mut self.median_time,
            &mut self.total,
            &mut self.weight,
            &mut self.position,
            &mut self.segwit_txs,
            &mut self.segwit_size,
            &mut self.segwit_weight,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.blockhash as &dyn AnyStoredVec,
            &self.coinbase_tag,
            &self.difficulty,
            &self.timestamp,
            &self.median_time,
            &self.total,
            &self.weight,
            &self.position,
            &self.segwit_txs,
            &self.segwit_size,
            &self.segwit_weight,
        ]
        .into_iter()
    }
}
