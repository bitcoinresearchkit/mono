//! Resume plumbing shared by the address and UTXO replays.

use bitview_plugin_indexer::Indexer;
use brk_error::{Error, Result};
use brk_types::{BlockHash, Cents, Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, ReadableVec};

/// Writer-owned replay state, retained only after a complete update succeeds.
pub struct LiveState<S> {
    pub end: usize,
    pub hash: Option<BlockHash>,
    pub version: Version,
    pub state: S,
    pub prices: Vec<Cents>,
}

/// Hash of the last block before `len`, identifying the chain a replay ends on.
pub fn tip_hash(indexer: &Indexer, len: usize) -> Option<BlockHash> {
    len.checked_sub(1)
        .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h)))
}

/// Validates every height-indexed output against `version`: `None` if any
/// computed version changed (replay from genesis), else the length all reach.
pub fn validate_outputs<'a>(
    vecs: impl ParallelIterator<Item = &'a mut dyn AnyStoredVec>,
    version: Version,
) -> Result<Option<usize>> {
    let (changed, len) = vecs
        .map(|v| Ok::<_, Error>((v.any_validate_computed_version_or_reset(version)?, v.len())))
        .try_reduce(
            || (false, usize::MAX),
            |a, b| Ok((a.0 || b.0, a.1.min(b.1))),
        )?;
    Ok((!changed).then_some(len))
}
