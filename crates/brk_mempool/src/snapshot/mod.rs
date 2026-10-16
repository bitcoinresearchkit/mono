pub mod block_stats;
pub mod builder;
pub mod cluster;
pub mod cpfp;
pub mod fees;
pub mod partition;
pub mod rebuilder;
pub mod snap_tx;
pub mod tx_index;

use std::{hash::Hasher, sync::Arc};

pub use block_stats::BlockStats;
use brk_error::{Error as QueryError, Result};
use brk_types::{
    FeeRate, MempoolBlock, NextBlockHash, PreciseRecommendedFees, RecommendedFees, Transaction,
    Txid,
};
use builder::KeyIndex;
pub use rebuilder::Rebuilder;
use rustc_hash::FxHasher;
use serde_json::to_vec;
pub use snap_tx::SnapTx;
pub use tx_index::TxIndex;

use crate::key::TxKey;

#[derive(Default)]
pub struct Snapshot {
    /// Dense per-tx data indexed by `TxIndex`. Each entry carries the
    /// linearized chunk rate plus parent/child adjacency.
    txs: Vec<SnapTx>,
    /// Projected blocks. `blocks[0]` is Core's `getblocktemplate`
    /// (Bitcoin Core's actual selection). The rest are greedy-packed
    /// by descending chunk rate, with a final overflow block.
    blocks: Vec<Vec<TxIndex>>,
    pub(crate) block_stats: Vec<BlockStats>,
    pub precise_fees: PreciseRecommendedFees,
    pub fees: RecommendedFees,
    min_fee: FeeRate,
    /// Content identity of the published template statistics and complete bodies.
    pub next_block_hash: NextBlockHash,
    key_to_idx: KeyIndex,
    template_transactions: Arc<[Arc<Transaction>]>,
    content_revision: u64,
    template_missing: bool,
}

impl Snapshot {
    /// `min_fee` is bitcoind's live `mempoolminfee`, the floor for
    /// every recommended-fee tier.
    fn build(
        txs: Vec<SnapTx>,
        blocks: Vec<Vec<TxIndex>>,
        key_to_idx: KeyIndex,
        min_fee: FeeRate,
    ) -> Self {
        let block_stats = BlockStats::for_blocks(&blocks, &txs);
        let precise_fees = fees::precise(&block_stats, min_fee);
        let fees = fees::whole(&block_stats, min_fee);
        Self {
            txs,
            blocks,
            block_stats,
            precise_fees,
            fees,
            min_fee,
            next_block_hash: NextBlockHash::ZERO,
            key_to_idx,
            template_transactions: Arc::from([]),
            content_revision: 0,
            template_missing: false,
        }
    }

    /// Freeze selected bodies before publication. Hash the complete public
    /// content, not only txids: prevout fills also change fee/sigop fields, which
    /// each body's `digests` entry follows.
    fn set_template(
        &mut self,
        bodies: Vec<Arc<Transaction>>,
        digests: &[u64],
        revision: u64,
        missing: bool,
    ) {
        let stats = self.template_stats();
        let mut hasher = FxHasher::default();
        hasher.write(b"template-v3");
        hasher.write(&to_vec(&stats).expect("template stats serialize"));
        for &digest in digests {
            hasher.write_u64(digest);
        }
        self.next_block_hash = NextBlockHash::new(hasher.finish());
        self.template_transactions = bodies.into();
        self.content_revision = revision;
        self.template_missing = missing;
    }

    fn tx(&self, idx: TxIndex) -> Option<&SnapTx> {
        self.txs.get(idx.as_usize())
    }

    fn idx_of(&self, key: &TxKey) -> Option<TxIndex> {
        self.key_to_idx.get(key).copied()
    }

    fn idx_of_txid(&self, txid: &Txid) -> Option<TxIndex> {
        let index = self.idx_of(&TxKey::from(txid))?;
        (self.tx(index)?.txid == *txid).then_some(index)
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

    /// Linearized chunk rate for a live tx. Recomputed each
    /// snapshot, package-aware (CPFP lifts apply), equals `fee/vsize`
    /// for singletons.
    pub(crate) fn chunk_rate_for(&self, txid: &Txid) -> Option<FeeRate> {
        let idx = self.idx_of_txid(txid)?;
        Some(self.txs[idx.as_usize()].chunk_rate)
    }

    /// Block zero's statistics, as published with its template.
    pub(crate) fn template_stats(&self) -> MempoolBlock {
        self.block_stats
            .first()
            .map(MempoolBlock::from)
            .unwrap_or_default()
    }

    pub(crate) fn template_transactions(&self) -> &Arc<[Arc<Transaction>]> {
        &self.template_transactions
    }

    pub(crate) fn content_revision(&self) -> u64 {
        self.content_revision
    }

    /// A default snapshot is not an observed empty mempool. A real publication
    /// always contains block zero, even when Core selected no transactions.
    pub(crate) fn ensure_projection(&self) -> Result<()> {
        if self.blocks.is_empty() || self.template_missing {
            return Err(QueryError::StateUpdating);
        }
        Ok(())
    }
}
