use crate::{Amount, Creations, util::invalid};
use std::io::Result;
/// A block's changes over the reader's decoded spend columns; no row is copied.
#[derive(Clone, Copy)]
pub struct BlockDiff<'a> {
    pub(crate) hash: [u8; 32],
    pub created: Amount,
    pub(crate) removed_total: Amount,
    origins: &'a [u32],
    counts: &'a [u64],
    sats: &'a [u64],
    correction: Option<(u32, Amount)>,
}
impl<'a> BlockDiff<'a> {
    /// Outputs spent by the block's inputs, per creation height, by ascending height.
    pub fn spent(&self) -> impl ExactSizeIterator<Item = (u32, Amount)> + Clone + 'a {
        (self.origins.iter().zip(self.counts).zip(self.sats))
            .map(|((&origin, &count), &sats)| (origin, Amount { sats, count }))
    }
    /// The output a BIP30 duplicate coinbase overwrote: it left the set without being spent.
    pub fn correction(&self) -> Option<(u32, Amount)> {
        self.correction
    }
    /// Everything that left the set: the spends and the correction.
    pub fn removed(&self) -> impl Iterator<Item = (u32, Amount)> + Clone + 'a {
        self.spent().chain(self.correction)
    }
    /// `spent` is the block's decoded spend record: its hash, total and rows (origins, counts,
    /// sats, of one length).
    pub(crate) fn new(
        height: usize,
        (hash, total): ([u8; 32], Amount),
        (origins, counts, sats): (&'a [u32], &'a [u64], &'a [u64]),
        output: &[u8],
    ) -> Result<Self> {
        let (output_hash, created, correction) = Creations::decode(height, output)?;
        if hash != output_hash {
            return Err(invalid("origin producer chain mismatch"));
        }
        let removed_total = correction.map_or(total, |(_, amount)| total + amount);
        Ok(Self {
            hash,
            created,
            removed_total,
            origins,
            counts,
            sats,
            correction,
        })
    }
}
