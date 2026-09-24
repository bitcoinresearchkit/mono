use std::cmp::Ordering;

use brk_error::Result;
use brk_types::{CentsCompact, Sats};

use super::AgeRangeUrpds;
use crate::UrpdRaw;

impl AgeRangeUrpds {
    pub(super) fn merge_sorted(
        left: &[(CentsCompact, Sats)],
        right: &[(CentsCompact, Sats)],
    ) -> Result<Vec<(CentsCompact, Sats)>> {
        let capacity = left
            .len()
            .saturating_add(right.len())
            .min(UrpdRaw::MAX_ENTRIES);
        let mut merged = Vec::with_capacity(capacity);
        let mut left_index = 0;
        let mut right_index = 0;

        while left_index < left.len() && right_index < right.len() {
            if merged.len() == UrpdRaw::MAX_ENTRIES {
                return Err(Self::invalid("aggregate exceeds snapshot entry limit"));
            }
            let left_entry = left[left_index];
            let right_entry = right[right_index];
            match left_entry.0.cmp(&right_entry.0) {
                Ordering::Less => {
                    merged.push(left_entry);
                    left_index += 1;
                }
                Ordering::Greater => {
                    merged.push(right_entry);
                    right_index += 1;
                }
                Ordering::Equal => {
                    let sats = u64::from(left_entry.1)
                        .checked_add(u64::from(right_entry.1))
                        .ok_or_else(|| Self::invalid("aggregate supply overflows"))?;
                    merged.push((left_entry.0, Sats::from(sats)));
                    left_index += 1;
                    right_index += 1;
                }
            }
        }

        if left.len() - left_index + right.len() - right_index > UrpdRaw::MAX_ENTRIES - merged.len()
        {
            return Err(Self::invalid("aggregate exceeds snapshot entry limit"));
        }
        merged.extend_from_slice(&left[left_index..]);
        merged.extend_from_slice(&right[right_index..]);
        Ok(merged)
    }
}

#[cfg(test)]
#[path = "../../tests/snapshots/aggregate.rs"]
mod tests;
