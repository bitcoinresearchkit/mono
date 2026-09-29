use crate::{Amount, Creations, Spends, util::invalid};
use std::io::Result;

/// Complete block changes, with distinct views of actual spends and all state removals.
#[derive(Default)]
pub struct BlockDiff {
    pub hash: [u8; 32],
    pub created: Amount,
    bytes: Vec<u8>,
    removed: Vec<(u32, Amount)>,
    spent_len: usize,
}

impl BlockDiff {
    /// Actual input spends, excluding historical output overwrites.
    pub fn spent(&self) -> &[(u32, Amount)] {
        &self.removed[..self.spent_len]
    }

    /// All removals from the UTXO state: actual spends followed by any overwrite.
    pub fn removed(&self) -> &[(u32, Amount)] {
        &self.removed
    }

    pub(crate) fn read(
        &mut self,
        height: usize,
        spends: &Spends,
        creations: &Creations,
    ) -> Result<()> {
        self.spent_len = 0;
        let (input_hash, _) = spends.read(height, &mut self.bytes, &mut self.removed)?;
        let (output_hash, created, correction) = creations.read(height)?;
        if input_hash != output_hash {
            return Err(invalid("origin producer chain mismatch"));
        }
        self.hash = input_hash;
        self.created = created;
        self.spent_len = self.removed.len();
        if let Some(correction) = correction {
            self.removed.push(correction);
        }
        Ok(())
    }
}
