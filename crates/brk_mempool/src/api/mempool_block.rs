//! A projected mempool block (block 0, Core's template): full content and incremental diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{
    MempoolBlockContent, MempoolBlockDiff, MempoolBlockDiffEntry, MempoolBlockHash, RetainedRun,
    Transaction, Txid,
};
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer};

use crate::{ReadOnlyState, ResolvedMempoolBlockDiff, Snapshot};

/// One immutable published template, retained for validation and body construction.
#[derive(Clone)]
pub struct MempoolBlockSource {
    snapshot: Arc<Snapshot>,
}

impl PartialEq for MempoolBlockSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.snapshot, &other.snapshot)
    }
}

impl Eq for MempoolBlockSource {}

impl MempoolBlockSource {
    /// Identity of the captured body; unavailable selections must not revalidate.
    pub fn hash(&self) -> Result<MempoolBlockHash> {
        self.snapshot.ensure_projection()?;
        Ok(self.snapshot.mempool_block_hash)
    }

    /// Build only after validation, without observing a newer publication.
    pub fn build(self) -> Result<MempoolBlockContent<TemplateTxs>> {
        Ok(MempoolBlockContent {
            hash: self.hash()?,
            stats: self.snapshot.template_stats(),
            transactions: TemplateTxs(self.snapshot.template_transactions().clone()),
        })
    }
}

impl ReadOnlyState {
    pub fn mempool_block_hash(&self) -> Result<MempoolBlockHash> {
        self.mempool_block_source().hash()
    }

    /// Capture the currently published template without cloning transaction bodies.
    #[must_use]
    pub fn mempool_block_source(&self) -> MempoolBlockSource {
        MempoolBlockSource {
            snapshot: self.snapshot().clone(),
        }
    }

    /// Validate and capture the historical side of one diff request.
    #[must_use]
    pub fn resolve_mempool_block_diff(
        &self,
        since: MempoolBlockHash,
    ) -> Option<ResolvedMempoolBlockDiff> {
        let past = self.historical_block0(since)?;
        let source = self.mempool_block_source();
        Some(ResolvedMempoolBlockDiff {
            since,
            past,
            source,
        })
    }
}

impl ResolvedMempoolBlockDiff {
    /// Build against the publication captured when history was resolved.
    pub fn build(self) -> Result<MempoolBlockDiff<TemplateOrder>> {
        let Self {
            since,
            past,
            source,
        } = self;
        let hash = source.hash()?;
        let bodies = source.snapshot.template_transactions().clone();
        let mut prior_index: FxHashMap<Txid, u32> = past
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
        Ok(MempoolBlockDiff {
            hash,
            since,
            stats: source.snapshot.template_stats(),
            order: TemplateOrder { bodies, entries },
            removed,
        })
    }
}

/// The published template's bodies, serialized in place as a JSON array.
pub struct TemplateTxs(Arc<[Arc<Transaction>]>);

impl Serialize for TemplateTxs {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(Arc::as_ref))
    }
}

/// The new template's entries, each new body a position in the published `bodies`.
pub struct TemplateOrder {
    bodies: Arc<[Arc<Transaction>]>,
    entries: Vec<MempoolBlockDiffEntry<usize>>,
}

impl Serialize for TemplateOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.entries.iter().map(|entry| match *entry {
            MempoolBlockDiffEntry::Retained(run) => MempoolBlockDiffEntry::Retained(run),
            MempoolBlockDiffEntry::New(position) => {
                MempoolBlockDiffEntry::New(&*self.bodies[position])
            }
        }))
    }
}
