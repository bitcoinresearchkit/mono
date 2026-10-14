use crate::{Amount, BlockDiff, util::invalid};
use std::io::Result;

#[derive(Clone, Debug, Default)]
pub struct State {
    amounts: Vec<Amount>,
    hash: [u8; 32],
    total: Amount,
}
impl State {
    pub(crate) fn new(amounts: Vec<Amount>, hash: [u8; 32]) -> Result<Self> {
        let total = amounts.iter().try_fold(Amount::default(), |total, &v| {
            if v.count == 0 && v.sats != 0 {
                return Err(invalid("supply without outputs"));
            }
            total.checked_add(v)
        })?;
        Ok(Self {
            amounts,
            hash,
            total,
        })
    }
    pub fn len(&self) -> usize {
        self.amounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.amounts.is_empty()
    }
    pub fn amounts(&self) -> &[Amount] {
        &self.amounts
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub(crate) fn total(&self) -> Amount {
        self.total
    }
    /// Applies one block of the validated producer chain: its rows were checked when produced.
    pub(crate) fn apply(&mut self, diff: &BlockDiff) {
        debug_assert_eq!(
            diff.removed()
                .fold(Amount::default(), |sum, (_, removed)| sum + removed),
            diff.removed_total,
            "origin total mismatch"
        );
        self.amounts.push(diff.created);
        for (origin, removed) in diff.removed() {
            debug_assert!(removed.count != 0, "removal without outputs");
            let current = &mut self.amounts[origin as usize];
            *current = *current - removed;
            debug_assert!(
                current.count != 0 || current.sats == 0,
                "remaining sats without outputs"
            );
        }
        self.hash = diff.hash;
        self.total = self.total + diff.created - diff.removed_total;
    }
}
