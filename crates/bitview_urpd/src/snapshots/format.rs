use std::ops::Range;

use bitview_cohort::{AGE_RANGE_COUNT, AgeRange};
use brk_error::Result;
use brk_types::Version;
use vecdb::Bytes;

use super::AgeRangeUrpds;

const MAGIC: [u8; 8] = *b"BRKARURP";
const FORMAT_VERSION: Version = Version::ONE.combine(Version::ONE);
const VERSION_OFFSET: usize = MAGIC.len();
const AGE_RANGE_COUNT_OFFSET: usize = VERSION_OFFSET + size_of::<Version>();
const OFFSETS_OFFSET: usize = AGE_RANGE_COUNT_OFFSET + size_of::<u32>();
pub(super) const HEADER_LEN: usize = OFFSETS_OFFSET + (AGE_RANGE_COUNT + 1) * size_of::<u64>();

impl AgeRangeUrpds {
    pub(super) fn new_buffer(capacity: usize) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(capacity.max(HEADER_LEN));
        buffer.resize(HEADER_LEN, 0);
        buffer[..MAGIC.len()].copy_from_slice(&MAGIC);
        buffer[VERSION_OFFSET..AGE_RANGE_COUNT_OFFSET]
            .copy_from_slice(FORMAT_VERSION.to_bytes().as_ref());
        buffer[AGE_RANGE_COUNT_OFFSET..OFFSETS_OFFSET]
            .copy_from_slice((AGE_RANGE_COUNT as u32).to_bytes().as_ref());
        Self::set_offset(&mut buffer, 0);
        buffer
    }

    pub(super) fn set_offset(buffer: &mut [u8], index: usize) {
        let start = OFFSETS_OFFSET + index * size_of::<u64>();
        let len = buffer.len() as u64;
        buffer[start..start + size_of::<u64>()].copy_from_slice(len.to_bytes().as_ref());
    }

    pub(super) fn ranges(header: &[u8], file_len: usize) -> Result<AgeRange<Range<usize>>> {
        if header.len() < HEADER_LEN {
            return Err(Self::invalid(format!(
                "header has {} bytes, expected {HEADER_LEN}",
                header.len()
            )));
        }
        if header[..MAGIC.len()] != MAGIC {
            return Err(Self::invalid("invalid magic"));
        }
        if Version::from_bytes(&header[VERSION_OFFSET..AGE_RANGE_COUNT_OFFSET])? != FORMAT_VERSION {
            return Err(Self::invalid("unsupported format version"));
        }
        if usize::try_from(u32::from_bytes(
            &header[AGE_RANGE_COUNT_OFFSET..OFFSETS_OFFSET],
        )?)
        .ok()
            != Some(AGE_RANGE_COUNT)
        {
            return Err(Self::invalid("unexpected age-range count"));
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
}
