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
    pub fn walk(
        client: &Client,
        anchor: Option<&BlockHash>,
        tip: Height,
    ) -> brk_error::Result<Self> {
        let start = match anchor {
            Some(hash) => Height::from((client.get_block_header_info(hash)?.height + 1) as u64),
            None => Height::ZERO,
        };
        let mut range = Self::between(client, start, tip)?;
        range.anchor = anchor.cloned();
        Ok(range)
    }

    pub fn between(client: &Client, start: Height, end: Height) -> brk_error::Result<Self> {
        if start > end {
            return Ok(Self {
                start,
                anchor: None,
                by_hash: FxHashMap::default(),
            });
        }
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

    pub fn len(&self) -> usize {
        self.by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_hash.is_empty()
    }

    #[inline]
    pub(crate) fn offset_of(&self, hash: &BlockHash) -> Option<u32> {
        self.by_hash.get(hash).copied()
    }

    /// `prev_hash` must match the canonical hash at `offset - 1`, or
    /// the anchor when `offset == 0`.
    #[inline]
    pub(crate) fn verify_prev(&self, offset: u32, prev_hash: &BlockHash) -> bool {
        match offset {
            0 => self.anchor.as_ref().is_none_or(|a| a == prev_hash),
            _ => self.offset_of(prev_hash) == Some(offset - 1),
        }
    }
}
