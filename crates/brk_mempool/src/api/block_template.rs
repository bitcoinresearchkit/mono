//! Projected next block: full template and incremental diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{
    BlockTemplate, BlockTemplateDiff, BlockTemplateDiffEntry, MempoolBlock, NextBlockHash, Txid,
};
use rustc_hash::FxHashMap;

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
    pub fn build(self) -> Result<BlockTemplate> {
        let hash = self.hash()?;
        Ok(BlockTemplate {
            hash,
            stats: self
                .snapshot
                .block_stats
                .first()
                .map(MempoolBlock::from)
                .unwrap_or_default(),
            transactions: self
                .snapshot
                .template_transactions()
                .iter()
                .map(|tx| tx.as_ref().clone())
                .collect(),
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
    pub fn build(self) -> Result<BlockTemplateDiff> {
        let Self {
            since,
            past,
            source,
        } = self;
        let hash = source.hash()?;
        let snap = &source.snapshot;
        let mut prior_index: FxHashMap<Txid, u32> = past
            .iter()
            .enumerate()
            .map(|(idx, tx)| (tx.txid, idx as u32))
            .collect();
        let mut order = Vec::with_capacity(snap.blocks.first().map_or(0, Vec::len));
        for tx in snap.template_transactions().iter() {
            let txid = tx.txid;
            match prior_index.remove(&txid) {
                Some(idx) if Arc::ptr_eq(tx, &past[idx as usize]) => {
                    order.push(BlockTemplateDiffEntry::Retained(idx))
                }
                _ => order.push(BlockTemplateDiffEntry::New(tx.as_ref().clone())),
            }
        }
        let removed = past
            .iter()
            .filter(|tx| prior_index.contains_key(&tx.txid))
            .map(|tx| tx.txid)
            .collect();
        Ok(BlockTemplateDiff {
            hash,
            since,
            order,
            removed,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/api/block_template.rs"]
mod tests;
