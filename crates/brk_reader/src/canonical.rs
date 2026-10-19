use brk_error::Result;
use brk_rpc::Client;
use brk_types::{BlockHash, Height};
use rustc_hash::FxHashMap;

/// Keyed on the full 32-byte hash: a prefix collision would
/// silently drop both blocks.
pub struct CanonicalRange {
    pub start: Height,
    anchor: Option<BlockHash>,
    by_hash: FxHashMap<BlockHash, u32>,
}

impl CanonicalRange {
    /// `None` when `anchor` is off the best chain: even an empty range
    /// (an orphan at the tip's height) must report the chain break.
    pub fn walk(client: &Client, anchor: Option<&BlockHash>, tip: Height) -> Result<Option<Self>> {
        let start = match anchor {
            Some(hash) => {
                let info = client.get_block_header_info(hash)?;
                if info.confirmations <= 0 {
                    return Ok(None);
                }
                Height::from((info.height + 1) as u64)
            }
            None => Height::ZERO,
        };
        let mut range = Self::between(client, start, tip)?;
        range.anchor = anchor.cloned();
        Ok(Some(range))
    }

    pub fn between(client: &Client, start: Height, end: Height) -> Result<Self> {
        let by_hash = client
            .get_block_hashes_range(*start, *end)?
            .into_iter()
            .enumerate()
            .map(|(i, h)| (h, i as u32))
            .collect();
        Ok(Self {
            start,
            anchor: None,
            by_hash,
        })
    }

    /// The block before the range, when it was resolved from one.
    pub fn anchor(&self) -> Option<&BlockHash> {
        self.anchor.as_ref()
    }

    pub fn len(&self) -> usize {
        self.by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_hash.is_empty()
    }

    #[inline]
    pub fn offset_of(&self, hash: &BlockHash) -> Option<u32> {
        self.by_hash.get(hash).copied()
    }

    /// `prev_hash` must match the canonical hash at `offset - 1`, or
    /// the anchor when `offset == 0`.
    #[inline]
    pub fn verify_prev(&self, offset: u32, prev_hash: &BlockHash) -> bool {
        match offset {
            0 => self.anchor.as_ref().is_none_or(|a| a == prev_hash),
            _ => self.offset_of(prev_hash) == Some(offset - 1),
        }
    }
}
