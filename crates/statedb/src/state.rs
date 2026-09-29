use crate::{Amount, util::invalid};
use std::io::Result;

#[derive(Clone, Debug, Default)]
pub struct State {
    pub(crate) amounts: Vec<Amount>,
    pub(crate) hash: [u8; 32],
    pub(crate) total: Amount,
}
impl State {
    pub fn new(amounts: Vec<Amount>, hash: [u8; 32]) -> Result<Self> {
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
    pub fn total(&self) -> Amount {
        self.total
    }
    /// A failed block never changes the state.
    pub(crate) fn apply(
        &mut self,
        hash: [u8; 32],
        created: Amount,
        rows: &[(u32, Amount)],
        scratch: &mut Vec<(usize, Amount)>,
    ) -> Result<()> {
        scratch.clear();
        let mut total = self.total.checked_add(created)?;
        // Apply provisionally, recording previous values. On failure undo in reverse order.
        self.amounts.push(created);
        let result = (|| {
            for &(origin, removed) in rows {
                let old = *self
                    .amounts
                    .get(origin as usize)
                    .ok_or_else(|| invalid("origin exceeds current block"))?;
                let new = old.checked_sub(removed)?;
                if new.count == 0 && new.sats != 0 {
                    return Err(invalid("remaining sats without outputs"));
                }
                total = total.checked_sub(removed)?;
                scratch.push((origin as usize, old));
                self.amounts[origin as usize] = new;
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
