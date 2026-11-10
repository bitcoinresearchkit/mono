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
    sync::{Arc, OnceLock},
};

pub use block_stats::BlockStats;
use brk_error::{Error as QueryError, Result};
use brk_types::{
    FeeRate, MempoolBlock, MempoolBlockHash, Transaction,
    Txid,
};
use builder::KeyIndex;
use rapidhash::RapidHashSet;
pub use rebuilder::Rebuilder;
use serde_json::to_vec;
pub use snap_tx::SnapTx;
pub use tx_index::TxIndex;

use crate::{JsonBody, json, key::TxKey};

#[derive(Default)]
pub struct Snapshot {
    /// Dense per-tx data indexed by `TxIndex`. Each entry carries the
    /// linearized chunk rate plus parent/child adjacency.
    txs: Vec<SnapTx>,
    /// Projected block 0: Core's `getblocktemplate` (Bitcoin Core's actual selection), or the
    /// packer's first block when Core gives none (`block0_packed`).
    block0: Vec<TxIndex>,
    pub(crate) block0_packed: bool,
    /// Every projected block's statistics: block 0's, then the package-aware projection's
    /// ([`partition`]). Empty only for a default snapshot.
    pub(crate) block_stats: Vec<BlockStats>,
    pub(crate) fees: fees::FeeView,
    /// Whether its view was complete (no template-only bodies, every listed transaction loaded
    /// or stuck): its fee tiers are its own. A rebuild reuses it only for a view alike.
    pub(crate) complete: bool,
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
    /// The gaps of the view it was built from (template-only bodies by their digests, in the
    /// template's order; listed txids not loaded): a rebuild from the same view, gaps included,
    /// reuses it.
    pub(crate) template_only: Box<[u64]>,
    pub(crate) unloaded: RapidHashSet<Txid>,
    /// The operator's fee deltas it ranked by: a rebuild under the same ones reuses it.
    pub(crate) fee_deltas: Box<[(Txid, i64)]>,
    /// The served bodies of the fee tiers and every block's stats, serialized by the
    /// first request after publication.
    fees_json: OnceLock<JsonBody>,
    precise_fees_json: OnceLock<JsonBody>,
    blocks_json: OnceLock<JsonBody>,
}

/// A projected block's bodies and complete identity: also all that diff history retains.
#[derive(Clone)]
pub(crate) struct BlockContent {
    pub hash: MempoolBlockHash,
    pub bodies: Arc<[Arc<Transaction>]>,
}

pub(crate) struct ServedBlock {
    pub content: BlockContent,
    /// The block served whole as JSON, built by the first request.
    pub json: OnceLock<Arc<[u8]>>,
}

impl Snapshot {
    /// `min_fee` is bitcoind's live `mempoolminfee`, the floor for
    /// every recommended-fee tier.
    /// `block0`: each member at its feerate there (its chunk's for the template's, the one it was
    /// packed at for the packer's: `block0_packed`). `held_fees`: an incomplete view's
    /// predecessor's tiers, and whether this view's block 0 is Core's whole template
    /// ([`fees::FeeView`]).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        txs: Vec<SnapTx>,
        block0: Vec<(TxIndex, FeeRate)>,
        block0_packed: bool,
        rest: &[Vec<(TxIndex, FeeRate)>],
        key_to_idx: KeyIndex,
        chunks: partition::Chunks,
        min_fee: FeeRate,
        held_fees: Option<(&fees::FeeView, bool)>,
    ) -> Self {
        let chunk_of = chunks.chunk_of(txs.len());
        let block_stats = BlockStats::for_blocks(&block0, rest, &txs);
        let block0 = block0.into_iter().map(|(index, _)| index).collect();
        let fees = fees::FeeView::new(
            &block_stats,
            held_fees,
            min_fee,
        );
        Self {
            txs,
            block0,
            block0_packed,
            block_stats,
            fees,
            complete: held_fees.is_none(),
            min_fee,
            key_to_idx,
            chunks,
            chunk_of,
            served: Vec::new(),
            content_revision: 0,
            template_missing: false,
            template_only: Box::default(),
            unloaded: RapidHashSet::default(),
            fee_deltas: Box::default(),
            fees_json: OnceLock::new(),
            precise_fees_json: OnceLock::new(),
            blocks_json: OnceLock::new(),
        }
    }

    /// mempool.space's `/api/v1/fees/recommended` body.
    pub(crate) fn fees_json(&self) -> Result<JsonBody> {
        self.ensure_fees()?;
        Ok(self.fees_json.get_or_init(|| json::body(&self.fees.whole)).clone())
    }

    /// mempool.space's `/api/v1/fees/precise` body.
    pub(crate) fn precise_fees_json(&self) -> Result<JsonBody> {
        self.ensure_fees()?;
        Ok(self
            .precise_fees_json
            .get_or_init(|| json::body(&self.fees.precise))
            .clone())
    }

    /// mempool.space's `/api/v1/fees/mempool-blocks` body.
    pub(crate) fn blocks_json(&self) -> Result<JsonBody> {
        self.ensure_projection()?;
        Ok(self
            .blocks_json
            .get_or_init(|| {
                let blocks: Vec<MempoolBlock> =
                    self.block_stats.iter().map(MempoolBlock::from).collect();
                json::body(&blocks)
            })
            .clone())
    }

    /// Freeze the served blocks' bodies (block 0 first), each with its digests, before
    /// publication. Each hash covers the complete public content, not only txids: prevout fills
    /// also change fee/sigop fields, which each body's digest follows.
    pub(crate) fn set_served(
        &mut self,
        blocks: Vec<(Vec<Arc<Transaction>>, Vec<u64>)>,
        revision: u64,
        missing: bool,
        previous: &[ServedBlock],
    ) {
        debug_assert!(!blocks.is_empty() && blocks.len() <= self.block_stats.len());
        self.served = blocks
            .into_iter()
            .enumerate()
            .map(|(index, (bodies, digests))| {
                let mut hasher = json::keyed_hasher();
                hasher.write_usize(index);
                hasher.write(&to_vec(&self.stats(index)).expect("block stats serialize"));
                for digest in digests {
                    hasher.write_u64(digest);
                }
                let hash = MempoolBlockHash::new(hasher.finish());
                let json = previous.get(index)
                    .filter(|block| block.content.hash == hash)
                    .and_then(|block| block.json.get())
                    .cloned()
                    .map_or_else(OnceLock::new, OnceLock::from);
                ServedBlock {
                    content: BlockContent { hash, bodies: bodies.into() },
                    json,
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

    /// Tiers from a complete view: its own, or one held over (as long as views stay incomplete,
    /// the last one standing; Core unread, the last publication's): only before the first
    /// complete view are there none. A block 0 missing a template member (which only its bodies
    /// wait for) leaves them standing.
    pub(crate) fn ensure_fees(&self) -> Result<()> {
        if !self.fees.complete {
            return Err(QueryError::StateUpdating);
        }
        Ok(())
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
