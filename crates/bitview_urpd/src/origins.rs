use crate::AgeRangeUrpds;
use bitview_cohort::AgeRange;
use brk_error::{Error, Result};
use brk_types::{Age, Cents, CentsCompact, Sats, Timestamp};
use statedb::State;

impl AgeRangeUrpds {
    /// Build canonical rounded age buckets from the remaining origin supplies.
    /// All/STH/LTH and model weights reuse these buckets and their existing rounding rules.
    pub fn from_origins(state: &State, prices: &[Cents], timestamps: &[Timestamp]) -> Result<Self> {
        let end = state.len();
        if end > prices.len() || end > timestamps.len() {
            return Err(Error::Internal("missing origin prices or timestamps"));
        }
        let mut groups: AgeRange<Vec<(CentsCompact, Sats)>> = AgeRange::default();
        if end == 0 {
            return Ok(Self::from_sorted_entries(|_| Vec::new()));
        }
        let current = timestamps[end - 1];
        for (h, amount) in state.amounts().iter().enumerate() {
            if amount.sats != 0 {
                groups
                    .get_mut(Age::new(current, timestamps[h]))
                    .push((prices[h].into(), Sats::new(amount.sats)));
            }
        }
        for entries in groups.iter_mut() {
            entries.sort_unstable_by_key(|v| v.0);
        }
        Ok(Self::from_sorted_entries(|id| {
            id.select(&groups).iter().copied()
        }))
    }
}
