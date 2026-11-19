//! A projected mempool block with its transactions in their light shape: whole and as a diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{
    MempoolBlockContent, MempoolBlockDiff, MempoolBlockDiffEntry, MempoolBlockHash,
    MempoolTxSummary, RetainedRun, Transaction, Txid,
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
    /// publication. A `since` history doesn't hold (aged out, minted by another server or before a
    /// restart) diffs from no copy, answered with `since` [`MempoolBlockHash::ZERO`]: every
    /// transaction new and none removed, so the client replaces its copy rather than patch it.
    #[must_use]
    pub fn resolve_mempool_block_diff(
        &self,
        source: MempoolBlockSource,
        since: MempoolBlockHash,
    ) -> ResolvedMempoolBlockDiff {
        ResolvedMempoolBlockDiff {
            past: self.historical_block(source.index, since),
            source,
        }
    }
}

impl ResolvedMempoolBlockDiff {
    /// Reuse the full diff's JSON when the client's base is absent from history.
    #[must_use]
    pub fn full_json(&self) -> Option<Arc<[u8]>> {
        if self.past.is_some() {
            return None;
        }
        let block = self.source.block();
        Some(
            block
                .full_diff_json
                .get_or_init(|| {
                    let diff = self.diff(BlockTxs(block.content.bodies.clone()), Vec::new());
                    serde_json::to_vec(&diff)
                        .expect("mempool block diffs serialize")
                        .into()
                })
                .clone(),
        )
    }

    /// Build against the publication captured when history was resolved.
    #[must_use]
    pub fn build(self) -> MempoolBlockDiff<BlockOrder> {
        let bodies = self.source.block().content.bodies.clone();
        let mut entries: Vec<MempoolBlockDiffEntry<usize>> = Vec::new();
        let removed = if self.past.as_ref().is_some_and(|past| Arc::ptr_eq(&past.bodies, &bodies)) {
            if !bodies.is_empty() {
                entries.push(MempoolBlockDiffEntry::Retained(RetainedRun(
                    0,
                    bodies.len() as u32,
                )));
            }
            Vec::new()
        } else {
            let past = self.past.as_ref().map_or(&[][..], |past| past.bodies.as_ref());
            let mut prior_index: RapidHashMap<Txid, u32> = past
                .iter()
                .enumerate()
                .map(|(idx, tx)| (tx.txid, idx as u32))
                .collect();
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
            past.iter()
                .filter(|tx| prior_index.contains_key(&tx.txid))
                .map(|tx| tx.txid)
                .collect()
        };
        self.diff(BlockOrder { bodies, entries }, removed)
    }

    fn diff<T>(&self, order: T, removed: Vec<Txid>) -> MempoolBlockDiff<T> {
        MempoolBlockDiff {
            hash: self.source.hash(),
            since: self.since(),
            stats: self.source.snapshot.stats(self.source.index),
            order,
            removed,
        }
    }
}

/// A published block's bodies, serialized in place as a JSON array of their light shapes.
struct BlockTxs(Arc<[Arc<Transaction>]>);

impl Serialize for BlockTxs {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|tx| MempoolTxSummary::from(&**tx)))
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
                MempoolBlockDiffEntry::New(MempoolTxSummary::from(&*self.bodies[position]))
            }
        }))
    }
}
