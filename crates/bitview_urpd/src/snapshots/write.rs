use std::{fs, path::Path};

use bitview_cohort::{AgeRange, AgeRangeId};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats};

use super::{AgeRangeUrpds, format::HEADER_LEN};
use crate::{UrpdRaw, rounded_entries};

impl AgeRangeUrpds {
    /// Round each age range's sorted prices and combine equal buckets.
    pub fn from_sorted_entries<I>(entries: impl Fn(AgeRangeId) -> I + Send + Sync) -> Self
    where
        I: IntoIterator<Item = (CentsCompact, Sats)>,
    {
        Self {
            entries: AgeRange::par_from_fn(|id| rounded_entries(entries(id)).collect()),
        }
    }

    pub fn write(&self, states_path: &Path, date: Date) -> Result<()> {
        let sections =
            AgeRange::par_try_from_fn(|id| UrpdRaw::serialize_iter(self.get(id).iter().copied()))?;
        let capacity = HEADER_LEN + sections.iter().map(Vec::len).sum::<usize>();
        if capacity > UrpdRaw::MAX_ENCODED_BYTES {
            return Err(Self::invalid("file exceeds snapshot limit"));
        }
        let mut buffer = Self::new_buffer(capacity);
        for (index, id) in AgeRangeId::ALL.iter().copied().enumerate() {
            buffer.extend_from_slice(id.select(&sections));
            Self::set_offset(&mut buffer, index + 1);
        }

        fs::create_dir_all(Self::dir(states_path))?;
        fs::write(Self::path(states_path, date), buffer)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/snapshots/write.rs"]
mod tests;
