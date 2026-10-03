use crate::{Amount, util::invalid};
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
    /// A failed block never changes the state.
    pub(crate) fn apply(
        &mut self,
        hash: [u8; 32],
        created: Amount,
        rows: impl Iterator<Item = (u32, Amount)>,
        removed_total: Amount,
        scratch: &mut Vec<(usize, Amount)>,
    ) -> Result<()> {
        scratch.clear();
        let total = self
            .total
            .checked_add(created)?
            .checked_sub(removed_total)?;
        // Apply provisionally, recording previous values. On failure undo in reverse order.
        self.amounts.push(created);
        let result = (|| {
            let mut actual = Amount::default();
            for (origin, removed) in rows {
                if removed.count == 0 {
                    return Err(invalid("removal without outputs"));
                }
                actual = actual.checked_add(removed)?;
                let current = self
                    .amounts
                    .get_mut(origin as usize)
                    .ok_or_else(|| invalid("origin exceeds current block"))?;
                let old = *current;
                let new = old.checked_sub(removed)?;
                if new.count == 0 && new.sats != 0 {
                    return Err(invalid("remaining sats without outputs"));
                }
                scratch.push((origin as usize, old));
                *current = new;
            }
            if actual != removed_total {
                return Err(invalid("origin total mismatch"));
            }
            Ok(())
        })();
        if let Err(error) = result {
            for &(i, old) in scratch.iter().rev() {
                self.amounts[i] = old;
            }
            self.amounts.pop();
            return Err(error);
        }
        self.hash = hash;
        self.total = total;
        Ok(())
    }
}
