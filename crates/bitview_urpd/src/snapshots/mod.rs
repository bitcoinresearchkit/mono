//! Indexed daily snapshots with independently compressed age ranges.
use bitview_cohort::{AgeRange, AgeRangeId};
use brk_error::Error;
use brk_types::{CentsCompact, Sats};

mod aggregate;
mod encoded;
mod format;
mod prune;
mod raw;
mod read;
mod write;

pub use encoded::EncodedAgeRangeUrpds;
pub use prune::prune_snapshots;

/// One day's independently compressed age-range URPDs in a single indexed file.
pub struct AgeRangeUrpds {
    entries: AgeRange<Vec<(CentsCompact, Sats)>>,
}

impl AgeRangeUrpds {
    pub fn get(&self, id: AgeRangeId) -> &[(CentsCompact, Sats)] {
        id.select(&self.entries)
    }

    pub fn iter(&self) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL.iter().copied().flat_map(|age| {
            self.get(age)
                .iter()
                .map(move |&(price, sats)| (age, price, sats))
        })
    }

    fn invalid(message: impl Into<String>) -> Error {
        Error::Deserialization(format!("AgeRangeUrpds: {}", message.into()))
    }
}
