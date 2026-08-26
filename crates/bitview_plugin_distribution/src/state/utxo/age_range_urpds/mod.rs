use std::{cmp::Ordering, ops::Range};

use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId};
use brk_error::{Error, Result};
use brk_types::{CentsCompact, Sats, Version};
use vecdb::{Bytes, ColumnId};

mod aggregate;
mod read;
mod write;

const DIR_NAME: &str = "utxos_age_range_urpds";
const MAGIC: [u8; 8] = *b"BRKARURP";
const FORMAT_VERSION: Version = Version::ONE.combine(AgeRangeId::VERSION);
const VERSION_OFFSET: usize = MAGIC.len();
const COLUMN_COUNT_OFFSET: usize = VERSION_OFFSET + size_of::<Version>();
const OFFSETS_OFFSET: usize = COLUMN_COUNT_OFFSET + size_of::<u32>();
const HEADER_LEN: usize = OFFSETS_OFFSET + (AGE_RANGE_COUNT + 1) * size_of::<u64>();

/// One day's independently compressed age-range URPDs in a single indexed file.
pub struct AgeRangeUrpds {
    entries: AgeRange<Vec<(CentsCompact, Sats)>>,
}

impl AgeRangeUrpds {
    fn merge_sorted(
        left: &[(CentsCompact, Sats)],
        right: &[(CentsCompact, Sats)],
    ) -> Vec<(CentsCompact, Sats)> {
        let mut merged = Vec::with_capacity(left.len() + right.len());
        let mut left_index = 0;
        let mut right_index = 0;

        while left_index < left.len() && right_index < right.len() {
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
                    merged.push((left_entry.0, left_entry.1 + right_entry.1));
                    left_index += 1;
                    right_index += 1;
                }
            }
        }

        merged.extend_from_slice(&left[left_index..]);
        merged.extend_from_slice(&right[right_index..]);
        merged
    }

    fn new_buffer(capacity: usize) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(capacity.max(HEADER_LEN));
        buffer.resize(HEADER_LEN, 0);
        buffer[..MAGIC.len()].copy_from_slice(&MAGIC);
        buffer[VERSION_OFFSET..COLUMN_COUNT_OFFSET]
            .copy_from_slice(FORMAT_VERSION.to_bytes().as_ref());
        buffer[COLUMN_COUNT_OFFSET..OFFSETS_OFFSET]
            .copy_from_slice((AGE_RANGE_COUNT as u32).to_bytes().as_ref());
        Self::set_offset(&mut buffer, 0);
        buffer
    }

    fn set_offset(buffer: &mut [u8], index: usize) {
        let start = OFFSETS_OFFSET + index * size_of::<u64>();
        let len = buffer.len() as u64;
        buffer[start..start + size_of::<u64>()].copy_from_slice(len.to_bytes().as_ref());
    }

    fn ranges(header: &[u8], file_len: usize) -> Result<AgeRange<Range<usize>>> {
        if header.len() < HEADER_LEN {
            return Err(Self::invalid(format!(
                "header has {} bytes, expected {HEADER_LEN}",
                header.len()
            )));
        }
        if header[..MAGIC.len()] != MAGIC {
            return Err(Self::invalid("invalid magic"));
        }
        if Version::from_bytes(&header[VERSION_OFFSET..COLUMN_COUNT_OFFSET])? != FORMAT_VERSION {
            return Err(Self::invalid("unsupported format version"));
        }
        if usize::try_from(u32::from_bytes(
            &header[COLUMN_COUNT_OFFSET..OFFSETS_OFFSET],
        )?)
        .ok()
            != Some(AGE_RANGE_COUNT)
        {
            return Err(Self::invalid("unexpected column count"));
        }

        let mut previous = Self::offset(header, 0)?;
        if previous != HEADER_LEN {
            return Err(Self::invalid("first section does not follow header"));
        }
        let ranges = AgeRange::try_from_fn(|id| {
            let start = Self::offset(header, id.index())?;
            let end = Self::offset(header, id.index() + 1)?;
            if start != previous || end < start || end > file_len {
                return Err(Self::invalid("invalid section offsets"));
            }
            previous = end;
            Ok(start..end)
        })?;
        if previous != file_len {
            return Err(Self::invalid("file length does not match final offset"));
        }
        Ok(ranges)
    }

    fn offset(header: &[u8], index: usize) -> Result<usize> {
        let start = OFFSETS_OFFSET + index * size_of::<u64>();
        usize::try_from(u64::from_bytes(&header[start..start + size_of::<u64>()])?)
            .map_err(|_| Self::invalid("section offset exceeds usize"))
    }

    fn invalid(message: impl Into<String>) -> Error {
        Error::Deserialization(format!("AgeRangeUrpds: {}", message.into()))
    }
}
