use bitcoin::block::Header;
use brk_error::{Error, Result};
use brk_rpc::Client;
use brk_types::{BlockHash, Height};
use rustc_hash::FxHashMap;

/// Keyed on the full 32-byte hash: a prefix collision would
/// silently drop both blocks.
pub struct CanonicalRange {
    pub start: Height,
    /// The block before the range and its header time.
    anchor: Option<(BlockHash, u32)>,
    by_hash: FxHashMap<BlockHash, u32>,
}

impl CanonicalRange {
    /// The best chain's blocks after `anchor` (from genesis when `None`) up to `end`. `None`
    /// when `anchor` is off the best chain: even an empty range (an orphan at the tip's height)
    /// must report the chain break.
    pub fn walk(client: &Client, anchor: Option<&BlockHash>, end: Height) -> Result<Option<Self>> {
        let (start, anchor) = match anchor {
            Some(hash) => {
                let info = client.get_block_header_info(hash)?;
                if info.confirmations <= 0 {
                    return Ok(None);
                }
                let start = Height::from((info.height + 1) as u64);
                (start, Some((*hash, info.time)))
            }
            None => (Height::ZERO, None),
        };
        let by_hash = client
            .get_block_hashes_range(*start, *end)?
            .into_iter()
            .enumerate()
            .map(|(i, h)| (h, i as u32))
            .collect();
        Ok(Some(Self {
            start,
            anchor,
            by_hash,
        }))
    }

    pub fn anchor(&self) -> Option<&BlockHash> {
        self.anchor.as_ref().map(|(hash, _)| hash)
    }

    pub fn anchor_time(&self) -> Option<u32> {
        self.anchor.as_ref().map(|&(_, time)| time)
    }

    pub fn len(&self) -> usize {
        self.by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_hash.is_empty()
    }

    /// The offset in the range of the block with this header, `None` for any other block. A
    /// block of the range that does not follow its canonical predecessor (the anchor, for the
    /// first) is an error: the range was resolved across a reorg.
    pub fn offset_of(&self, header: &Header) -> Result<Option<u32>> {
        let Some(&offset) = self.by_hash.get(&BlockHash::from(header.block_hash())) else {
            return Ok(None);
        };
        let prev = BlockHash::from(header.prev_blockhash);
        let follows = match offset {
            0 => self.anchor().is_none_or(|anchor| *anchor == prev),
            _ => self.by_hash.get(&prev) == Some(&(offset - 1)),
        };
        if !follows {
            return Err(Error::Internal("canonical batch stitched across a reorg"));
        }
        Ok(Some(offset))
    }
}
