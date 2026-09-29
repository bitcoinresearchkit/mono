use crate::rounded_entries;
use bitview_cohort::{AgeRange, AgeRangeId};
use brk_types::{CentsCompact, Sats};

/// Rounded price buckets for one block, grouped by the age of their remaining supply.
pub struct AgeRangeUrpds {
    entries: AgeRange<Vec<(CentsCompact, Sats)>>,
}
impl AgeRangeUrpds {
    pub fn from_sorted_entries<I>(entries: impl Fn(AgeRangeId) -> I + Send + Sync) -> Self
    where
        I: IntoIterator<Item = (CentsCompact, Sats)>,
    {
        Self {
            entries: AgeRange::par_from_fn(|id| rounded_entries(entries(id)).collect()),
        }
    }
    pub fn get(&self, id: AgeRangeId) -> &[(CentsCompact, Sats)] {
        id.select(&self.entries)
    }
    pub fn iter(&self) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL
            .iter()
            .copied()
            .flat_map(|age| self.get(age).iter().map(move |&(p, s)| (age, p, s)))
    }
}
