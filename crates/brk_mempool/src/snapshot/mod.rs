pub mod block_stats;
pub mod builder;
pub mod cluster;
pub mod cpfp;
pub mod fees;
pub mod partition;
pub mod rebuilder;
pub mod snap_tx;
pub mod tx_index;

use std::{
    hash::Hasher,
    sync::Arc,
    time::{Duration, Instant},
};

pub use block_stats::BlockStats;
use brk_error::{Error as QueryError, Result};
use brk_types::{
    FeeRate, MempoolBlock, MempoolBlockHash, PreciseRecommendedFees, RecommendedFees, Transaction,
    Txid,
};
use builder::KeyIndex;
use rapidhash::RapidHashSet;
pub use rebuilder::Rebuilder;
use rustc_hash::FxHasher;
use serde_json::to_vec;
pub use snap_tx::SnapTx;
pub use tx_index::TxIndex;

use crate::key::TxKey;

/// How long an incomplete view holds a complete one's fee tiers over: about a block. Past it the
/// fee endpoints answer `StateUpdating` until a complete view returns.
const HELD_FEES_MAX: Duration = Duration::from_secs(600);

#[derive(Default)]
pub struct Snapshot {
    /// Dense per-tx data indexed by `TxIndex`. Each entry carries the
    /// linearized chunk rate plus parent/child adjacency.
    txs: Vec<SnapTx>,
    /// Projected block 0: Core's `getblocktemplate` (Bitcoin Core's actual selection).
    block0: Vec<TxIndex>,
    /// Every projected block's statistics: block 0's, then the package-aware projection's
    /// ([`partition`]). Empty only for a default snapshot.
    pub(crate) block_stats: Vec<BlockStats>,
    pub precise_fees: PreciseRecommendedFees,
    pub fees: RecommendedFees,
    /// When a complete view last stood behind `fees` and `precise_fees` (this one's build, or for
    /// tiers held over, the last tick that saw a complete view), and whether this snapshot holds
    /// them over (built itself from part of the pool, still loading or under a stale template, its
    /// own blocks after block 0 would quote too low). None before the first complete view.
    pub(crate) fees_from: Option<Instant>,
    pub(crate) fees_held: bool,
    min_fee: FeeRate,
    key_to_idx: KeyIndex,
    /// Every component's chunks as linearized for this snapshot, and each transaction's chunk:
    /// CPFP answers read them instead of linearizing again.
    chunks: partition::Chunks,
    chunk_of: Vec<u32>,
    /// The blocks served with their transactions: block 0 and the projection's, but a last one
    /// holding more than a block (what's left, mempool.space's catch-all).
    served: Vec<ServedBlock>,
    content_revision: u64,
    template_missing: bool,
    /// The gaps of the view it was built from (template-only txids, listed txids not loaded):
    /// a rebuild from the same view, gaps included, reuses it.
    pub(crate) template_only: RapidHashSet<Txid>,
    pub(crate) unloaded: RapidHashSet<Txid>,
    /// The operator's fee deltas it ranked by: a rebuild under the same ones reuses it.
    pub(crate) fee_deltas: Box<[(Txid, i64)]>,
}

/// A projected block's bodies in block order, frozen at publication, and its content identity.
#[derive(Clone)]
pub(crate) struct ServedBlock {
    pub hash: MempoolBlockHash,
    pub bodies: Arc<[Arc<Transaction>]>,
}

impl Snapshot {
    /// `min_fee` is bitcoind's live `mempoolminfee`, the floor for
    /// every recommended-fee tier.
    pub(crate) fn build(
        txs: Vec<SnapTx>,
        block0: Vec<TxIndex>,
        rest: &[Vec<(TxIndex, FeeRate)>],
        key_to_idx: KeyIndex,
        chunks: partition::Chunks,
        min_fee: FeeRate,
    ) -> Self {
        let chunk_of = chunks.chunk_of(txs.len());
        let block_stats = BlockStats::for_blocks(&block0, rest, &txs);
        let precise_fees = fees::precise(&block_stats, min_fee);
        let fees = fees::whole(&block_stats, min_fee);
        Self {
            txs,
            block0,
            block_stats,
            precise_fees,
            fees,
            fees_from: Some(Instant::now()),
            fees_held: false,
            min_fee,
            key_to_idx,
            chunks,
            chunk_of,
            served: Vec::new(),
            content_revision: 0,
            template_missing: false,
            template_only: RapidHashSet::default(),
            unloaded: RapidHashSet::default(),
            fee_deltas: Box::default(),
        }
    }

    /// Freeze the served blocks' bodies (block 0 first), each with its digests, before
    /// publication. Each hash covers the complete public content, not only txids: prevout fills
    /// also change fee/sigop fields, which each body's digest follows.
    pub(crate) fn set_served(
        &mut self,
        blocks: Vec<(Vec<Arc<Transaction>>, Vec<u64>)>,
        revision: u64,
        missing: bool,
    ) {
        debug_assert!(!blocks.is_empty() && blocks.len() <= self.block_stats.len());
        self.served = blocks
            .into_iter()
            .enumerate()
            .map(|(index, (bodies, digests))| {
                let mut hasher = FxHasher::default();
                hasher.write(b"mempool-block-v4");
                hasher.write_usize(index);
                hasher.write(&to_vec(&self.stats(index)).expect("block stats serialize"));
                for digest in digests {
                    hasher.write_u64(digest);
                }
                ServedBlock {
                    hash: MempoolBlockHash::new(hasher.finish()),
                    bodies: bodies.into(),
                }
            })
            .collect();
        self.content_revision = revision;
        self.template_missing = missing;
    }

    fn idx_of_txid(&self, txid: &Txid) -> Option<TxIndex> {
        let index = *self.key_to_idx.get(&TxKey::from(txid))?;
        (self.txs[index.as_usize()].txid == *txid).then_some(index)
    }

    /// Txids of `blocks[0]` (Core's `getblocktemplate` selection),
    /// in template order. Empty for a default snapshot.
    pub fn block0_txids(&self) -> impl Iterator<Item = Txid> + '_ {
        self.block0.iter().map(|idx| self.txs[idx.as_usize()].txid)
    }

    /// Linearized chunk rate for a live tx. Recomputed each
    /// snapshot, package-aware (CPFP lifts apply), equals `fee/vsize`
    /// for singletons.
    pub(crate) fn chunk_rate_for(&self, txid: &Txid) -> Option<FeeRate> {
        let idx = self.idx_of_txid(txid)?;
        Some(self.txs[idx.as_usize()].chunk_rate)
    }

    /// Block `index`'s statistics, as published with its bodies.
    pub(crate) fn stats(&self, index: usize) -> MempoolBlock {
        MempoolBlock::from(&self.block_stats[index])
    }

    /// Block `index` with its transactions, once a projection is published: `None` past the
    /// projection, and for a last block holding more than a block.
    pub(crate) fn served(&self, index: usize) -> Result<Option<&ServedBlock>> {
        self.ensure_projection()?;
        Ok(self.served.get(index))
    }

    /// Every served block, block 0 first.
    pub(crate) fn served_blocks(&self) -> &[ServedBlock] {
        &self.served
    }

    pub(crate) fn content_revision(&self) -> u64 {
        self.content_revision
    }

    /// [`Self::ensure_projection`], with tiers from a complete view: its own, or one held over
    /// for at most [`HELD_FEES_MAX`].
    pub(crate) fn ensure_fees(&self) -> Result<()> {
        self.ensure_projection()?;
        match self.fees_from {
            Some(from) if !self.fees_held || from.elapsed() <= HELD_FEES_MAX => Ok(()),
            _ => Err(QueryError::StateUpdating),
        }
    }

    /// A default snapshot is not an observed empty mempool. A real publication
    /// always contains block zero, even when Core selected no transactions.
    pub(crate) fn ensure_projection(&self) -> Result<()> {
        if self.block_stats.is_empty() || self.template_missing {
            return Err(QueryError::StateUpdating);
        }
        Ok(())
    }
}
