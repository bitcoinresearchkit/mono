//! Projected next block: full template and incremental diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{
    BlockTemplate, BlockTemplateDiff, BlockTemplateDiffEntry, NextBlockHash, Transaction, Txid,
};
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer};

use crate::{ReadOnlyState, ResolvedBlockTemplateDiff, Snapshot};

/// One immutable published template, retained for validation and body construction.
#[derive(Clone)]
pub struct BlockTemplateSource {
    snapshot: Arc<Snapshot>,
}

impl PartialEq for BlockTemplateSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.snapshot, &other.snapshot)
    }
}

impl Eq for BlockTemplateSource {}

impl BlockTemplateSource {
    /// Identity of the captured body; unavailable selections must not revalidate.
    pub fn hash(&self) -> Result<NextBlockHash> {
        self.snapshot.ensure_projection()?;
        Ok(self.snapshot.next_block_hash)
    }

    /// Build only after validation, without observing a newer publication.
    pub fn build(self) -> Result<BlockTemplate<TemplateTxs>> {
        Ok(BlockTemplate {
            hash: self.hash()?,
            stats: self.snapshot.template_stats(),
            transactions: TemplateTxs(self.snapshot.template_transactions().clone()),
        })
    }
}

impl ReadOnlyState {
    pub fn next_block_hash(&self) -> Result<NextBlockHash> {
        self.block_template_source().hash()
    }

    /// Capture the currently published template without cloning transaction bodies.
    #[must_use]
    pub fn block_template_source(&self) -> BlockTemplateSource {
        BlockTemplateSource {
            snapshot: self.snapshot(),
        }
    }

    /// Validate and capture the historical side of one diff request.
    #[must_use]
    pub fn resolve_block_template_diff(
        &self,
        since: NextBlockHash,
    ) -> Option<ResolvedBlockTemplateDiff> {
        let past = self.historical_block0(since)?;
        let source = self.block_template_source();
        Some(ResolvedBlockTemplateDiff {
            since,
            past,
            source,
        })
    }
}

impl ResolvedBlockTemplateDiff {
    /// Build against the publication captured when history was resolved.
    pub fn build(self) -> Result<BlockTemplateDiff<TemplateOrder>> {
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
        let entries = bodies
            .iter()
            .enumerate()
            .map(|(position, tx)| match prior_index.remove(&tx.txid) {
                Some(idx) if Arc::ptr_eq(tx, &past[idx as usize]) => {
                    BlockTemplateDiffEntry::Retained(idx)
                }
                _ => BlockTemplateDiffEntry::New(position),
            })
            .collect();
        let removed = past
            .iter()
            .filter(|tx| prior_index.contains_key(&tx.txid))
            .map(|tx| tx.txid)
            .collect();
        Ok(BlockTemplateDiff {
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
    entries: Vec<BlockTemplateDiffEntry<usize>>,
}

impl Serialize for TemplateOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.entries.iter().map(|entry| match *entry {
            BlockTemplateDiffEntry::Retained(idx) => BlockTemplateDiffEntry::Retained(idx),
            BlockTemplateDiffEntry::New(position) => {
                BlockTemplateDiffEntry::New(&*self.bodies[position])
            }
        }))
    }
}
