use std::ptr;

use memmap2::MmapRaw;

/// Writes `data` at `offset` into the mmap. Panics on out-of-bounds.
///
/// # Safety
/// The destination must be exclusively accessible for the duration of the copy.
#[inline]
pub(crate) unsafe fn write_to_mmap(mmap: &MmapRaw, offset: usize, data: &[u8]) {
    let end = offset
        .checked_add(data.len())
        .expect("offset + data.len() overflow");
    assert!(end <= mmap.len());

    unsafe {
        let ptr = mmap.as_mut_ptr();
        ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(offset), data.len());
    }
}
