use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    BlkPosition, BlockHash, CoinbaseTag, Height, StoredF64, StoredU32, StoredU64, Timestamp,
    Version, Weight,
};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, CachedVec, Database, ImportableVec, PcoVec, Rw, Stamp,
    StorageMode, TypedVec, VecIndex, VecValue, WritableVec,
};

use crate::parallel_import;

fn truncate_cached<I, T, V>(vec: &mut CachedVec<V>, index: I, stamp: Stamp) -> Result<()>
where
    I: VecIndex,
    T: VecValue,
    V: TypedVec<I = I, T = T> + WritableVec<I, T>,
{
    let truncates = index.to_usize() < vec.len();
    vec.inner.truncate_if_needed_with_stamp(index, stamp)?;
    if truncates {
        vec.invalidate();
    }
    Ok(())
}

#[derive(Traversable)]
pub struct BlocksVecs<M: StorageMode = Rw> {
    pub blockhash: CachedVec<M::Stored<BytesVec<Height, BlockHash>>>,
    pub coinbase_tag: M::Stored<BytesVec<Height, CoinbaseTag>>,
    #[traversable(wrap = "difficulty", rename = "value")]
    pub difficulty: M::Stored<PcoVec<Height, StoredF64>>,
    /// Unix timestamp in seconds associated with the indexed block or time
    /// period. Block-header timestamps are not guaranteed to increase between
    /// consecutive heights.
    #[traversable(wrap = "time")]
    pub timestamp: CachedVec<M::Stored<PcoVec<Height, Timestamp>>>,
    #[traversable(wrap = "size", rename = "base")]
    pub total: M::Stored<PcoVec<Height, StoredU64>>,
    #[traversable(wrap = "weight", rename = "base")]
    pub weight: M::Stored<PcoVec<Height, Weight>>,
    #[traversable(hidden)]
    pub position: M::Stored<PcoVec<Height, BlkPosition>>,
    pub segwit_txs: M::Stored<PcoVec<Height, StoredU32>>,
    pub segwit_size: M::Stored<PcoVec<Height, StoredU64>>,
    pub segwit_weight: M::Stored<PcoVec<Height, Weight>>,
}

impl BlocksVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (
            blockhash,
            coinbase_tag,
            difficulty,
            timestamp,
            total,
            weight,
            position,
            segwit_txs,
            segwit_size,
            segwit_weight,
        ) = parallel_import! {
            blockhash = BytesVec::forced_import(db, "blockhash", version),
            coinbase_tag = BytesVec::forced_import(db, "coinbase_tag", version),
            difficulty = PcoVec::forced_import(db, "difficulty", version),
            timestamp = PcoVec::forced_import(db, "timestamp", version),
            total_size = PcoVec::forced_import(db, "total_size", version),
            weight = PcoVec::forced_import(db, "block_weight", version),
            position = PcoVec::forced_import(db, "block_position", version),
            segwit_txs = PcoVec::forced_import(db, "segwit_txs", version),
            segwit_size = PcoVec::forced_import(db, "segwit_size", version),
            segwit_weight = PcoVec::forced_import(db, "segwit_weight", version),
        };
        Ok(Self {
            blockhash: CachedVec::wrap(blockhash),
            coinbase_tag,
            difficulty,
            timestamp: CachedVec::wrap(timestamp),
            total,
            weight,
            position,
            segwit_txs,
            segwit_size,
            segwit_weight,
        })
    }

    pub fn truncate(&mut self, height: Height, stamp: Stamp) -> Result<()> {
        truncate_cached(&mut self.blockhash, height, stamp)?;
        self.coinbase_tag
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.difficulty
            .truncate_if_needed_with_stamp(height, stamp)?;
        truncate_cached(&mut self.timestamp, height, stamp)?;
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
            &mut self.blockhash.inner as &mut dyn AnyStoredVec,
            &mut self.coinbase_tag,
            &mut self.difficulty,
            &mut self.timestamp.inner,
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
            &self.blockhash.inner as &dyn AnyStoredVec,
            &self.coinbase_tag,
            &self.difficulty,
            &self.timestamp.inner,
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

#[cfg(test)]
mod tests {
    use vecdb::ReadableVec;

    use super::*;

    #[test]
    fn truncate_cached_invalidates_same_length_cache() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let inner =
            PcoVec::<Height, Timestamp>::forced_import(&db, "timestamp", Version::ONE).unwrap();
        let mut timestamps = CachedVec::wrap(inner);

        for timestamp in [10_u32, 20, 30] {
            timestamps.inner.push(Timestamp::from(timestamp));
        }
        assert_eq!(timestamps.collect(), [10_u32, 20, 30].map(Timestamp::from));

        truncate_cached(&mut timestamps, Height::from(1_usize), Stamp::from(0_u64)).unwrap();
        timestamps.inner.push(Timestamp::from(200_u32));
        timestamps.inner.push(Timestamp::from(300_u32));

        assert_eq!(
            timestamps.collect(),
            [10_u32, 200, 300].map(Timestamp::from)
        );
    }
}
