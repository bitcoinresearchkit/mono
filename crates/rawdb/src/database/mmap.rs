use std::ptr;

use memmap2::MmapRaw;

/// Copies a validated range into the mapping.
///
/// # Safety
/// The destination range must be mapped and exclusively accessible.
#[inline]
pub(crate) unsafe fn write_to_mmap(mmap: &MmapRaw, offset: usize, data: &[u8]) {
    debug_assert!(offset <= mmap.len() && data.len() <= mmap.len() - offset);

    unsafe {
        let ptr = mmap.as_mut_ptr();
        ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(offset), data.len());
    }
}
