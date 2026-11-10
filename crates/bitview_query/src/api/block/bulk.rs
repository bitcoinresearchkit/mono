use std::sync::Arc;

use bitcoin::hashes::sha256;
use bitview_types::BlockBulk;
use brk_types::Height;
use vecdb::VecIndex;

use super::{ResolvedBlocks, StaleBlocks, blocks_v1_identity};
use crate::{Error, Query, Result};

/// Public mempool.space bulk bound, inclusive of both endpoints.
const MAX_BLOCKS_BULK: u32 = 10;

/// An exact subrange of a pinned block window, and one published stale-block view.
/// Carries no price data: bulk has no price field.
pub struct ResolvedBlocksBulk {
    blocks: ResolvedBlocks,
    from: Height,
    stale: Arc<StaleBlocks>,
}

fn range_count(from: Height, to: Height) -> Result<u32> {
    let distance = u32::from(to).checked_sub(u32::from(from))
        .ok_or_else(|| Error::InvalidParam("Parameter 'to' must be a higher block height than 'from'".into()))?;
    if distance >= MAX_BLOCKS_BULK {
        return Err(Error::InvalidParam(format!("You can only query {MAX_BLOCKS_BULK} blocks at once.")));
    }
    Ok(distance + 1)
}

impl Query {
    /// Resolve and validate before any analytics or blk-file reads. Omitted `to` is the
    /// same publication's safe tip; an explicit future `to` never returns a partial range.
    pub fn resolve_blocks_bulk(&self, from: Height, to: Option<Height>) -> Result<ResolvedBlocksBulk> {
        let count = to.map(|to| range_count(from, to)).transpose()?;
        // With no endpoint yet, retain at most the public ten-height window. Capture below
        // starts at `from`, so none of the earlier heights' data is read or formatted.
        let blocks = self.blocks_window(to, count.unwrap_or(MAX_BLOCKS_BULK))?;
        let end = match to {
            Some(to) if blocks.last_height() != Some(to) => {
                return Err(Error::NotFound("Block not found".into()));
            }
            Some(to) => to,
            None => blocks.last_height().ok_or(Error::StateUpdating)?,
        };
        if count.is_none() {
            range_count(from, end)?;
        }
        debug_assert!(from.to_usize() >= blocks.range().0);
        Ok(ResolvedBlocksBulk { blocks, from, stale: self.stale_blocks() })
    }
}

impl ResolvedBlocksBulk {
    pub fn is_settled(&self) -> bool {
        self.blocks.is_settled()
    }

    /// The anchor identifies all canonical rows; orphan observations are captured once.
    /// The caller includes `from` and its wire/formula version in the HTTP tag.
    pub fn identity(&self) -> sha256::Hash {
        let (_, end) = self.blocks.range();
        let orphans = (self.from.to_usize()..end)
            .flat_map(|height| self.stale.at(Height::from(height)));
        blocks_v1_identity(self.blocks.anchor(), &[], orphans)
    }

    /// Build ascending rows from the shared facts capture, dropping read guards before I/O.
    pub fn build(self, query: &Query) -> Result<Vec<BlockBulk>> {
        let pin = self.blocks.read_pin(query);
        let (_, end) = self.blocks.range();
        let build = query.capture_blocks_bulk_range(self.from.to_usize(), end, &pin, self.stale)?;
        drop(pin);
        drop(self.blocks);
        build()
    }
}
