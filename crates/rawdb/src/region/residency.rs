#[cfg(unix)]
use std::sync::OnceLock;

#[cfg(unix)]
use libc::{_SC_PAGESIZE, mincore, sysconf};
use memmap2::MmapRaw;

pub(crate) const MMAP_RESIDENCY_MIN_BYTES: usize = 128 * 1024;
#[cfg(unix)]
const SAMPLE_BYTES: usize = 16 * 1024 * 1024;
#[cfg(unix)]
static VM_PAGE_SIZE: OnceLock<Option<usize>> = OnceLock::new();

/// The VM page size, if it divides the sampling window.
#[cfg(unix)]
fn vm_page_size() -> Option<usize> {
    *VM_PAGE_SIZE.get_or_init(|| {
        // SAFETY: this query has no pointer or ownership requirements.
        usize::try_from(unsafe { sysconf(_SC_PAGESIZE) })
            .ok()
            .filter(|&size| size > 0 && SAMPLE_BYTES.is_multiple_of(size))
    })
}

/// Samples the first, last, and one page per window of a mapped range.
/// Empty in-bounds ranges need no probing. Unsupported page geometry, missing
/// pages, and probe errors return false.
#[cfg(unix)]
pub(crate) fn is_range_resident(mmap: &MmapRaw, offset: usize, len: usize) -> bool {
    if len == 0 {
        return offset <= mmap.len();
    }
    let Some(page_size) = vm_page_size() else {
        return false;
    };
    let start = offset / page_size * page_size;
    let Some(absolute_end) = offset.checked_add(len) else {
        return false;
    };
    let Some(end) = absolute_end.checked_next_multiple_of(page_size) else {
        return false;
    };
    let sampled_len = end - start;

    if end > mmap.len() {
        return false;
    }

    let is_resident = |at: usize| {
        let mut state = 0u8;
        // SAFETY: at is page-aligned and the checked mapping contains a full
        // page. mincore writes one status byte for the single page queried.
        let result = unsafe {
            mincore(
                mmap.as_ptr().add(at).cast_mut().cast(),
                page_size,
                (&raw mut state).cast(),
            )
        };
        result == 0 && state & 1 != 0
    };

    if !is_resident(start) || !is_resident(end - page_size) {
        return false;
    }

    let chunks = sampled_len.div_ceil(SAMPLE_BYTES);
    let mut state = (start as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(sampled_len as u64);
    for chunk in 0..chunks {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;

        let chunk_start = chunk * SAMPLE_BYTES;
        let chunk_len = (sampled_len - chunk_start).min(SAMPLE_BYTES);
        let pages = chunk_len / page_size;
        let page = state as usize % pages;
        if !is_resident(start + chunk_start + page * page_size) {
            return false;
        }
    }

    true
}

/// Whether every page of a page-aligned mapped range is in memory (one probe for the whole range).
/// Unsupported page geometry and probe errors return false.
#[cfg(unix)]
pub(crate) fn is_fully_resident(mmap: &MmapRaw, offset: usize, len: usize) -> bool {
    let Some(page_size) = vm_page_size() else {
        return false;
    };
    if !offset.is_multiple_of(page_size)
        || offset.checked_add(len).is_none_or(|end| end > mmap.len())
    {
        return false;
    }
    let mut states = vec![0u8; len.div_ceil(page_size)];
    // SAFETY: the checked, page-aligned range lies within the mapping, and mincore writes one status
    // byte per page into a buffer sized for every page of it.
    let result = unsafe {
        mincore(
            mmap.as_ptr().add(offset).cast_mut().cast(),
            len,
            states.as_mut_ptr().cast(),
        )
    };
    result == 0 && states.iter().all(|state| state & 1 != 0)
}

#[cfg(not(unix))]
pub(crate) fn is_range_resident(_mmap: &MmapRaw, _offset: usize, _len: usize) -> bool {
    false
}
