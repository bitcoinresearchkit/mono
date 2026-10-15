//! Projected next block: full template and incremental diff.

use std::sync::Arc;

use brk_error::Result;

use brk_types::{NextBlockHash, Transaction, Txid};
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer, ser::SerializeStruct};

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
    pub fn build(self) -> Result<BlockTemplateJson> {
        Ok(BlockTemplateJson {
            hash: self.hash()?,
            snapshot: self.snapshot,
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
    pub fn build(self) -> Result<BlockTemplateDiffJson> {
        let Self {
            since,
            past,
            source,
        } = self;
        let hash = source.hash()?;
        let snapshot = source.snapshot;
        let mut prior_index: FxHashMap<Txid, u32> = past
            .iter()
            .enumerate()
            .map(|(idx, tx)| (tx.txid, idx as u32))
            .collect();
        let order = snapshot
            .template_transactions()
            .iter()
            .enumerate()
            .map(|(position, tx)| match prior_index.remove(&tx.txid) {
                Some(idx) if Arc::ptr_eq(tx, &past[idx as usize]) => Slot::Retained(idx),
                _ => Slot::New(position),
            })
            .collect();
        let removed = past
            .iter()
            .filter(|tx| prior_index.contains_key(&tx.txid))
            .map(|tx| tx.txid)
            .collect();
        Ok(BlockTemplateDiffJson {
            hash,
            since,
            order,
            removed,
            snapshot,
        })
    }
}

/// Serializes as [`BlockTemplate`] straight from the published bodies, without copying them.
///
/// [`BlockTemplate`]: brk_types::BlockTemplate
pub struct BlockTemplateJson {
    hash: NextBlockHash,
    snapshot: Arc<Snapshot>,
}

impl Serialize for BlockTemplateJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let bodies = self.snapshot.template_transactions();
        let mut template = serializer.serialize_struct("BlockTemplate", 3)?;
        template.serialize_field("hash", &self.hash)?;
        template.serialize_field("stats", &self.snapshot.template_stats())?;
        template.serialize_field("transactions", &Seq(|| bodies.iter().map(Arc::as_ref)))?;
        template.end()
    }
}

/// Serializes as [`BlockTemplateDiff`], new bodies straight from the published ones.
///
/// [`BlockTemplateDiff`]: brk_types::BlockTemplateDiff
pub struct BlockTemplateDiffJson {
    hash: NextBlockHash,
    since: NextBlockHash,
    order: Vec<Slot>,
    removed: Vec<Txid>,
    snapshot: Arc<Snapshot>,
}

/// A [`brk_types::BlockTemplateDiffEntry`] whose new body is a position in the template.
enum Slot {
    Retained(u32),
    New(usize),
}

#[derive(Serialize)]
#[serde(untagged)]
enum SlotJson<'a> {
    Retained(u32),
    New(&'a Transaction),
}

impl Serialize for BlockTemplateDiffJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let bodies = self.snapshot.template_transactions();
        let order = || {
            self.order.iter().map(|slot| match *slot {
                Slot::Retained(idx) => SlotJson::Retained(idx),
                Slot::New(position) => SlotJson::New(&bodies[position]),
            })
        };
        let mut diff = serializer.serialize_struct("BlockTemplateDiff", 4)?;
        diff.serialize_field("hash", &self.hash)?;
        diff.serialize_field("since", &self.since)?;
        diff.serialize_field("order", &Seq(order))?;
        diff.serialize_field("removed", &self.removed)?;
        diff.end()
    }
}

/// A JSON array of what the iterator `F` makes yields.
struct Seq<F>(F);

impl<F, I> Serialize for Seq<F>
where
    F: Fn() -> I,
    I: Iterator<Item: Serialize>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq((self.0)())
    }
}
