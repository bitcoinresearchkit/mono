use crate::{BlockDiff, Creations, Spends, journal_reader::JournalReader, util::invalid};
use std::io::Result;

/// Sequential block diffs over one validated prefix, with bounded read windows and no state:
/// each block must continue the chain of the one before it.
pub struct Diffs<'a> {
    spends: JournalReader<'a>,
    creations: JournalReader<'a>,
    height: usize,
    end: usize,
    /// The last block's hash, which the next block must continue (unchecked before the first).
    hash: Option<[u8; 32]>,
}

impl<'a> Diffs<'a> {
    pub(crate) fn new(
        height: usize,
        hash: Option<[u8; 32]>,
        end: usize,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Self> {
        if height > end || end > spends.end().min(creations.end()) {
            return Err(invalid("invalid replay range"));
        }
        Ok(Self {
            spends: spends.cursor(end),
            creations: creations.cursor(end),
            height,
            end,
            hash,
        })
    }

    /// The hash of the last block returned, `None` before the first.
    pub fn hash(&self) -> Option<[u8; 32]> {
        self.hash
    }

    /// The next block's changes; None marks the published end.
    pub fn advance(&mut self) -> Result<Option<BlockDiff<'_>>> {
        if self.height == self.end {
            return Ok(None);
        }
        let height = self.height;
        let (parent, spent) = self.spends.read(height)?;
        // A rewind racing this replay (readers hold their own file handles) can hand it another
        // chain's block: the read must continue the previous block's chain.
        if let Some(hash) = self.hash
            && parent.get(..32) != Some(hash.as_slice())
        {
            return Err(invalid("replay state does not match producer chain"));
        }
        let diff = BlockDiff::decode(height, spent, self.creations.read(height)?.1)?;
        self.height += 1;
        self.hash = Some(diff.hash);
        Ok(Some(diff))
    }
}
