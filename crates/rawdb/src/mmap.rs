use std::fs::File;

use memmap2::{MmapMut, MmapOptions};

#[inline]
pub fn create_mmap(file: &File) -> crate::Result<MmapMut> {
    Ok(unsafe { MmapOptions::new().map_mut(file)? })
}

/// Writes `data` at `offset` into the mmap. Panics on out-of-bounds.
#[inline]
pub fn write_to_mmap(mmap: &MmapMut, offset: usize, data: &[u8]) {
    let end = offset
        .checked_add(data.len())
        .expect("offset + data.len() overflow");
    assert!(end <= mmap.len());

    unsafe {
        let ptr = mmap.as_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(offset), data.len());
    }
}
