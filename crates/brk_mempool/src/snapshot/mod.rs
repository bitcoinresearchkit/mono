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
    FeeRate, MempoolBlock, NextBlockHash, RecommendedFees, Transaction, Txid, TxidPrefix,
};
use builder::PrefixIndex;
pub use rebuilder::Rebuilder;
use rustc_hash::FxHasher;
use serde_json::to_vec;
pub use snap_tx::SnapTx;
pub use tx_index::TxIndex;

#[derive(Default)]
pub struct Snapshot {
    /// Dense per-tx data indexed by `TxIndex`. Each entry carries the
    /// linearized chunk rate plus parent/child adjacency.
    pub txs: Vec<SnapTx>,
    /// Projected blocks. `blocks[0]` is Core's `getblocktemplate`
    /// (Bitcoin Core's actual selection). The rest are greedy-packed
    /// by descending chunk rate, with a final overflow block.
    pub blocks: Vec<Vec<TxIndex>>,
    pub(crate) block_stats: Vec<BlockStats>,
    pub fees: RecommendedFees,
    min_fee: FeeRate,
    /// Content identity of the published template statistics and complete bodies.
    pub next_block_hash: NextBlockHash,
    prefix_to_idx: PrefixIndex,
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
        prefix_to_idx: PrefixIndex,
        min_fee: FeeRate,
    ) -> Self {
        let block_stats = BlockStats::for_blocks(&blocks, &txs);
        let fees = fees::compute(&block_stats, min_fee);
        Self {
            txs,
            blocks,
            block_stats,
            fees,
            min_fee,
            next_block_hash: NextBlockHash::ZERO,
            prefix_to_idx,
            template_transactions: Arc::from([]),
            content_revision: 0,
            template_missing: false,
        }
    }

    /// Freeze selected bodies before publication. Hash the complete public
    /// content, not only txids: prevout fills also change fee/sigop fields.
    fn set_template(&mut self, bodies: Vec<Arc<Transaction>>, revision: u64, missing: bool) {
        let stats = self
            .block_stats
            .first()
            .map(MempoolBlock::from)
            .unwrap_or_default();
        let borrowed: Vec<&Transaction> = bodies.iter().map(Arc::as_ref).collect();
        let bytes = to_vec(&(stats, borrowed)).expect("template fields serialize");
        let mut hasher = FxHasher::default();
        hasher.write(b"template-v2");
        hasher.write(&bytes);
        self.next_block_hash = NextBlockHash::new(hasher.finish());
        self.template_transactions = bodies.into();
        self.content_revision = revision;
        self.template_missing = missing;
    }

    fn tx(&self, idx: TxIndex) -> Option<&SnapTx> {
        self.txs.get(idx.as_usize())
    }

    fn idx_of(&self, prefix: &TxidPrefix) -> Option<TxIndex> {
        self.prefix_to_idx.get(prefix).copied()
    }

    fn idx_of_txid(&self, txid: &Txid) -> Option<TxIndex> {
        let index = self.idx_of(&TxidPrefix::from(txid))?;
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

#[cfg(test)]
#[path = "../../tests/unit/snapshot.rs"]
mod tests;
