use crate::{Amount, Creations, Spends, util::invalid};
use std::io::Result;
/// A block view into the cursor's read window; no decoded row buffer or copy.
#[derive(Clone, Copy)]
pub struct BlockDiff<'a> {
    pub(crate) hash: [u8; 32],
    pub created: Amount,
    pub(crate) removed_total: Amount,
    rows: &'a [u8],
    correction: Option<(u32, Amount)>,
}
impl<'a> BlockDiff<'a> {
    fn spent(&self) -> impl ExactSizeIterator<Item = (u32, Amount)> + Clone + '_ {
        Spends::rows(self.rows)
    }
    pub fn removed(&self) -> impl Iterator<Item = (u32, Amount)> + Clone + '_ {
        self.spent().chain(self.correction)
    }
    pub(crate) fn decode(height: usize, input: &'a [u8], output: &[u8]) -> Result<Self> {
        let (hash, total) = Spends::header(input)?;
        let (output_hash, created, correction) = Creations::decode(height, output)?;
        if hash != output_hash {
            return Err(invalid("origin producer chain mismatch"));
        }
        let removed_total = if let Some((_, amount)) = correction {
            total.checked_add(amount)?
        } else {
            total
        };
        Ok(Self {
            hash,
            created,
            removed_total,
            rows: &input[48..],
            correction,
        })
    }
}
