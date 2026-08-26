mod block_stats;
mod builder;
mod cluster;
mod cpfp;
mod fees;
mod partition;
mod rebuilder;
mod snap_tx;
mod tx_index;

pub use block_stats::BlockStats;
pub use cluster::Cluster;
pub use rebuilder::Rebuilder;
pub use snap_tx::SnapTx;
pub use tx_index::TxIndex;

use builder::PrefixIndex;
use fees::Fees;
use partition::Partitioner;

use std::hash::{Hash, Hasher};

use brk_types::{FeeRate, NextBlockHash, RecommendedFees, Txid, TxidPrefix};
use rustc_hash::FxHasher;

#[derive(Default)]
pub struct Snapshot {
    /// Dense per-tx data indexed by `TxIndex`. Each entry carries the
    /// linearized chunk rate plus parent/child adjacency.
    pub txs: Vec<SnapTx>,
    /// Projected blocks. `blocks[0]` is Core's `getblocktemplate`
    /// (Bitcoin Core's actual selection). The rest are greedy-packed
    /// by descending chunk rate, with a final overflow block.
    pub blocks: Vec<Vec<TxIndex>>,
    pub block_stats: Vec<BlockStats>,
    pub fees: RecommendedFees,
    min_fee: FeeRate,
    /// Content hash of the projected next block. Same value as the
    /// mempool `ETag`.
    pub next_block_hash: NextBlockHash,
    prefix_to_idx: PrefixIndex,
}

impl Snapshot {
    /// `min_fee` is bitcoind's live `mempoolminfee`, the floor for
    /// every recommended-fee tier.
    fn build(
        txs: Vec<SnapTx>,
        blocks: Vec<Vec<TxIndex>>,
        prefix_to_idx: PrefixIndex,
        min_fee: FeeRate,
    ) -> Self {
        let block_stats = BlockStats::for_blocks(&blocks, &txs);
        let fees = Fees::compute(&block_stats, min_fee);
        let next_block_hash = Self::hash_next_block(&blocks, &txs);
        Self {
            txs,
            blocks,
            block_stats,
            fees,
            min_fee,
            next_block_hash,
            prefix_to_idx,
        }
    }

    /// Content tag over block 0 in template order. Hashes txids, not
    /// `TxIndex` slots, because slot assignment is per-cycle.
    fn hash_next_block(blocks: &[Vec<TxIndex>], txs: &[SnapTx]) -> NextBlockHash {
        let Some(block) = blocks.first() else {
            return NextBlockHash::ZERO;
        };
        let mut hasher = FxHasher::default();
        for idx in block {
            txs[idx.as_usize()].txid.hash(&mut hasher);
        }
        NextBlockHash::new(hasher.finish())
    }

    pub fn tx(&self, idx: TxIndex) -> Option<&SnapTx> {
        self.txs.get(idx.as_usize())
    }

    pub fn idx_of(&self, prefix: &TxidPrefix) -> Option<TxIndex> {
        self.prefix_to_idx.get(prefix).copied()
    }

    /// Txids of `blocks[0]` (Core's `getblocktemplate` selection),
    /// in template order. Empty for a default snapshot.
    pub fn block0_txids(&self) -> impl Iterator<Item = Txid> + '_ {
        self.blocks
            .first()
            .into_iter()
            .flatten()
            .map(|idx| self.txs[idx.as_usize()].txid)
    }

    /// Linearized chunk rate for a live tx by prefix. Recomputed each
    /// snapshot, package-aware (CPFP lifts apply), equals `fee/vsize`
    /// for singletons.
    pub fn chunk_rate_for(&self, prefix: &TxidPrefix) -> Option<FeeRate> {
        let idx = self.idx_of(prefix)?;
        Some(self.txs[idx.as_usize()].chunk_rate)
    }

    /// Test-only: stitch a snapshot from `(prefix, chunk_rate)` pairs
    /// without running the full builder.
    #[cfg(test)]
    pub(crate) fn for_test_with_chunk_rates(entries: &[(TxidPrefix, FeeRate, Txid)]) -> Self {
        use brk_types::{Sats, VSize, Weight};
        use smallvec::SmallVec;

        let mut prefix_to_idx = PrefixIndex::default();
        let mut txs = Vec::with_capacity(entries.len());
        for (i, (prefix, rate, txid)) in entries.iter().enumerate() {
            prefix_to_idx.insert(*prefix, TxIndex::from(i));
            txs.push(SnapTx {
                txid: *txid,
                fee: Sats::ZERO,
                vsize: VSize::from(0u64),
                weight: Weight::from(0u64),
                size: 0,
                chunk_rate: *rate,
                parents: SmallVec::new(),
                children: SmallVec::new(),
            });
        }
        Self {
            txs,
            blocks: vec![],
            block_stats: vec![],
            fees: RecommendedFees::default(),
            min_fee: FeeRate::default(),
            next_block_hash: NextBlockHash::ZERO,
            prefix_to_idx,
        }
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::Hash;
    use brk_types::{Sats, VSize, Weight};
    use smallvec::SmallVec;

    use super::*;

    fn snap_tx(seed: u8) -> SnapTx {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        SnapTx {
            txid: Txid::from(bitcoin::Txid::from_byte_array(bytes)),
            fee: Sats::from(1_234u64),
            vsize: VSize::from(100u64),
            weight: Weight::from(400u64),
            size: 100,
            chunk_rate: FeeRate::from((Sats::from(1_234u64), VSize::from(100u64))),
            parents: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    #[test]
    fn next_block_hash_is_deterministic_across_runs() {
        let txs = vec![snap_tx(1), snap_tx(2), snap_tx(3)];
        let blocks = vec![vec![
            TxIndex::from(0usize),
            TxIndex::from(1usize),
            TxIndex::from(2usize),
        ]];
        let h1 = Snapshot::hash_next_block(&blocks, &txs);
        let h2 = Snapshot::hash_next_block(&blocks, &txs);
        assert_eq!(h1, h2);
    }

    #[test]
    fn next_block_hash_changes_with_block0_membership() {
        let txs = vec![snap_tx(1), snap_tx(2), snap_tx(3)];
        let two_member = vec![vec![TxIndex::from(0usize), TxIndex::from(1usize)]];
        let three_member = vec![vec![
            TxIndex::from(0usize),
            TxIndex::from(1usize),
            TxIndex::from(2usize),
        ]];
        assert_ne!(
            Snapshot::hash_next_block(&two_member, &txs),
            Snapshot::hash_next_block(&three_member, &txs),
        );
    }

    #[test]
    fn next_block_hash_changes_with_block0_order() {
        // hash_next_block hashes txids in template order: reordering
        // block 0 must produce a different hash.
        let txs = vec![snap_tx(1), snap_tx(2), snap_tx(3)];
        let forward = vec![vec![
            TxIndex::from(0usize),
            TxIndex::from(1usize),
            TxIndex::from(2usize),
        ]];
        let reversed = vec![vec![
            TxIndex::from(2usize),
            TxIndex::from(1usize),
            TxIndex::from(0usize),
        ]];
        assert_ne!(
            Snapshot::hash_next_block(&forward, &txs),
            Snapshot::hash_next_block(&reversed, &txs),
        );
    }

    #[test]
    fn empty_blocks_hash_is_zero() {
        let txs = vec![snap_tx(1)];
        let blocks: Vec<Vec<TxIndex>> = vec![];
        assert_eq!(
            Snapshot::hash_next_block(&blocks, &txs),
            NextBlockHash::ZERO
        );
    }
}
