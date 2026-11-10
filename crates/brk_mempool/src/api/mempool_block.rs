//! A projected mempool block with its transactions: full content and incremental diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{
    MempoolBlockContent, MempoolBlockDiff, MempoolBlockDiffEntry, MempoolBlockHash, RetainedRun,
    Transaction, Txid,
};
use rapidhash::RapidHashMap;
use serde::{Serialize, Serializer};

use crate::{ReadOnlyState, ResolvedMempoolBlockDiff, Snapshot, snapshot::ServedBlock};

/// One block of an immutable publication, retained for validation and body construction.
#[derive(Clone)]
pub struct MempoolBlockSource {
    snapshot: Arc<Snapshot>,
    index: usize,
}

impl PartialEq for MempoolBlockSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.snapshot, &other.snapshot) && self.index == other.index
    }
}

impl Eq for MempoolBlockSource {}

impl MempoolBlockSource {
    fn block(&self) -> &ServedBlock {
        &self.snapshot.served_blocks()[self.index]
    }

    /// Identity of the captured body.
    #[must_use]
    pub fn hash(&self) -> MempoolBlockHash {
        self.block().content.hash
    }

    /// The captured block as JSON, never a newer publication's: reusing an equal predecessor's encoding.
    #[must_use]
    pub fn json(&self) -> Arc<[u8]> {
        let block = self.block();
        block
            .json
            .get_or_init(|| {
                let content = MempoolBlockContent {
                    hash: block.content.hash,
                    stats: self.snapshot.stats(self.index),
                    transactions: BlockTxs(block.content.bodies.clone()),
                };
                serde_json::to_vec(&content)
                    .expect("mempool blocks serialize")
                    .into()
            })
            .clone()
    }
}

impl ReadOnlyState {
    /// Block `index` of the current publication, captured without cloning transaction bodies:
    /// `None` when it isn't served, past the projection or a last block holding more than a block.
    pub fn mempool_block_source(&self, index: usize) -> Result<Option<MempoolBlockSource>> {
        let snapshot = self.snapshot();
        Ok(snapshot.served(index)?.map(|_| MempoolBlockSource {
            snapshot: snapshot.clone(),
            index,
        }))
    }

    /// How many blocks the current publication projects, served or not.
    pub fn projected_blocks(&self) -> Result<usize> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.block_stats.len())
    }

    /// The currently published template's transactions, in template order.
    #[must_use]
    pub fn template_transactions(&self) -> Arc<[Arc<Transaction>]> {
        self.snapshot()
            .served_blocks()
            .first()
            .map_or_else(|| Arc::from([]), |block| block.content.bodies.clone())
    }

    /// Capture the past side of one diff request against `source`, a block of this
    /// publication: `None` once history no longer holds that block at `since`.
    #[must_use]
    pub fn resolve_mempool_block_diff(
        &self,
        source: MempoolBlockSource,
        since: MempoolBlockHash,
    ) -> Option<ResolvedMempoolBlockDiff> {
        let past = self.historical_block(source.index, since)?;
        Some(ResolvedMempoolBlockDiff {
            since,
            past,
            source,
        })
    }
}

impl ResolvedMempoolBlockDiff {
    /// Build against the publication captured when history was resolved.
    #[must_use]
    pub fn build(self) -> MempoolBlockDiff<BlockOrder> {
        let Self {
            since,
            past,
            source,
        } = self;
        let block = source.block();
        let (hash, bodies) = (block.content.hash, block.content.bodies.clone());
        let mut prior_index: RapidHashMap<Txid, u32> = past
            .iter()
            .enumerate()
            .map(|(idx, tx)| (tx.txid, idx as u32))
            .collect();
        let mut entries: Vec<MempoolBlockDiffEntry<usize>> = Vec::new();
        for (position, tx) in bodies.iter().enumerate() {
            match prior_index.remove(&tx.txid) {
                Some(idx) if Arc::ptr_eq(tx, &past[idx as usize]) => {
                    // A kept transaction continuing the previous run extends it.
                    if let Some(MempoolBlockDiffEntry::Retained(run)) = entries.last_mut()
                        && run.0 + run.1 == idx
                    {
                        run.1 += 1;
                    } else {
                        entries.push(MempoolBlockDiffEntry::Retained(RetainedRun(idx, 1)));
                    }
                }
                _ => entries.push(MempoolBlockDiffEntry::New(position)),
            }
        }
        let removed = past
            .iter()
            .filter(|tx| prior_index.contains_key(&tx.txid))
            .map(|tx| tx.txid)
            .collect();
        MempoolBlockDiff {
            hash,
            since,
            stats: source.snapshot.stats(source.index),
            order: BlockOrder { bodies, entries },
            removed,
        }
    }
}

/// A published block's bodies, serialized in place as a JSON array.
pub struct BlockTxs(Arc<[Arc<Transaction>]>);

impl Serialize for BlockTxs {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(Arc::as_ref))
    }
}

/// The new block's entries, each new body a position in the published `bodies`.
pub struct BlockOrder {
    bodies: Arc<[Arc<Transaction>]>,
    entries: Vec<MempoolBlockDiffEntry<usize>>,
}

impl Serialize for BlockOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.entries.iter().map(|entry| match *entry {
            MempoolBlockDiffEntry::Retained(run) => MempoolBlockDiffEntry::Retained(run),
            MempoolBlockDiffEntry::New(position) => {
                MempoolBlockDiffEntry::New(&*self.bodies[position])
            }
        }))
    }
}
