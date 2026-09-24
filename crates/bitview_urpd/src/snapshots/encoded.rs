use std::{ops::Range, os::unix::fs::FileExt, path::Path};

use bitview_cohort::{AgeRange, UTXOAggregateId};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats};

use super::AgeRangeUrpds;
use crate::{UrpdRaw, distribution::checked_supply};

/// Owned compressed input for one aggregate, captured under publication protection.
pub struct EncodedAgeRangeUrpds {
    data: Vec<u8>,
    ranges: AgeRange<Range<usize>>,
    start: usize,
    id: UTXOAggregateId,
}

impl EncodedAgeRangeUrpds {
    /// The selected encoded sections in their aggregation order.
    pub fn sections(&self) -> impl Iterator<Item = &[u8]> {
        self.id.age_range_ids().iter().map(|id| {
            let range = id.select(&self.ranges);
            &self.data[range.start - self.start..range.end - self.start]
        })
    }

    /// Decode and validate the selected aggregate without building a map.
    pub fn decode_entries(&self) -> Result<Vec<(CentsCompact, Sats)>> {
        // Decode one section at a time, including All, so captured requests do
        // not retain the expanded maps for every age range simultaneously.
        let entries = self.sections().try_fold(Vec::new(), |left, section| {
            let right = UrpdRaw::deserialize_entries(section)?;
            if left.is_empty() {
                return Ok(right);
            }
            if right.is_empty() {
                return Ok(left);
            }
            AgeRangeUrpds::merge_sorted(&left, &right)
        })?;
        checked_supply(entries.iter().map(|(_, sats)| u64::from(*sats)))?;
        Ok(entries)
    }
}

impl AgeRangeUrpds {
    /// Capture encoded aggregate input while holding the producer's publication guard.
    pub fn read_aggregate_encoded(
        states_path: &Path,
        id: UTXOAggregateId,
        date: Date,
    ) -> Result<EncodedAgeRangeUrpds> {
        if id == UTXOAggregateId::All {
            let data = Self::read_bytes(states_path, date)?;
            let ranges = Self::ranges(&data, data.len())?;
            return Ok(EncodedAgeRangeUrpds {
                data,
                ranges,
                start: 0,
                id,
            });
        }

        let path = Self::path(states_path, date);
        let (file, ranges) = Self::open(&path)?;
        let ids = id.age_range_ids();
        debug_assert!(
            ids.windows(2)
                .all(|pair| pair[0].index() + 1 == pair[1].index())
        );
        let first = ids.first().expect("aggregate contains an age range");
        let last = ids.last().expect("aggregate contains an age range");
        let start = first.select(&ranges).start;
        let end = last.select(&ranges).end;
        let mut data = vec![0; end - start];
        file.read_exact_at(&mut data, start as u64)?;

        Ok(EncodedAgeRangeUrpds {
            data,
            ranges,
            start,
            id,
        })
    }
}
