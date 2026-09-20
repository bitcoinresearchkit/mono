use std::{fs::File, path::Path, slice};

use memmap2::{MmapOptions, MmapRaw};
use parking_lot::Mutex;

use crate::{Error, RegionMetadata, Result, region::metadata::SIZE_OF_REGION_METADATA};

use super::{locked_file::open_locked_file, mmap::write_to_mmap};

/// Owns metadata slots, their mapping, and synchronization to disk.
pub(crate) struct MetadataFile {
    file: File,
    mmap: MmapRaw,
    dirty: Mutex<bool>,
}

impl MetadataFile {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let file = open_locked_file(path)?;
        let mmap = MmapRaw::map_raw(&file)?;
        if !mmap.len().is_multiple_of(SIZE_OF_REGION_METADATA) {
            return Err(Error::CorruptedMetadata(format!(
                "regions file size {} is not a multiple of {}",
                mmap.len(),
                SIZE_OF_REGION_METADATA
            )));
        }
        Ok(Self {
            file,
            mmap,
            // A reopened mapping may include unsynced writes from its previous
            // owner. The first flush must establish a data + metadata boundary.
            dirty: Mutex::new(true),
        })
    }

    /// Exclusive access prevents writes while the initial slots are decoded.
    pub(crate) fn slots(
        &mut self,
    ) -> impl ExactSizeIterator<Item = Result<Option<RegionMetadata>>> + '_ {
        // SAFETY: exclusive access keeps the mapping stable and prevents writes.
        let bytes = unsafe { slice::from_raw_parts(self.mmap.as_ptr(), self.mmap.len()) };
        bytes
            .as_chunks::<SIZE_OF_REGION_METADATA>()
            .0
            .iter()
            .map(RegionMetadata::from_bytes)
    }

    pub(crate) fn reserve(&mut self, slots: usize) -> Result<()> {
        let len = slots * SIZE_OF_REGION_METADATA;
        if len <= self.mmap.len() {
            return Ok(());
        }
        let file_len = self.file.metadata()?.len() as usize;
        let target_len = if file_len < len {
            len.max(file_len.saturating_mul(2))
        } else {
            file_len
        };
        if target_len != file_len {
            self.file.set_len(target_len as u64)?;
        }
        // Retry a previous failed remap even if the file already grew.
        if self.mmap.len() != target_len {
            self.mmap = MmapRaw::map_raw(&self.file)?;
        }
        Ok(())
    }

    /// The database must flush data and metadata before discarding empty slots.
    pub(crate) fn truncate(&mut self, slots: usize) -> Result<()> {
        let len = slots * SIZE_OF_REGION_METADATA;
        if self.file.metadata()?.len() > len as u64 {
            let mmap = MmapOptions::new().len(len).map_raw(&self.file)?;
            self.file.set_len(len as u64)?;
            self.mmap = mmap;
            *self.dirty.get_mut() = true;
            self.file.sync_all()?;
            *self.dirty.get_mut() = false;
        }
        Ok(())
    }

    pub(crate) fn write(&self, index: usize, meta: &RegionMetadata) {
        self.write_slot(index, &meta.to_bytes());
    }

    /// IDs never change within a live slot. Length/growth updates only need its
    /// three numeric bounds, not another zero-fill and copy of the whole slot.
    pub(crate) fn update_bounds(&self, index: usize, meta: &RegionMetadata) {
        self.write_slot(index, &meta.bounds_bytes());
    }

    pub(crate) fn clear(&self, index: usize) {
        self.write_slot(index, &[0; SIZE_OF_REGION_METADATA]);
    }

    fn write_slot(&self, index: usize, bytes: &[u8]) {
        assert!(bytes.len() <= SIZE_OF_REGION_METADATA);
        let mut dirty = self.dirty.lock();
        // SAFETY: the mutex serializes writes/flushes; shared access excludes
        // remapping and decoding. No reference over other slots is formed.
        unsafe { write_to_mmap(&self.mmap, index * SIZE_OF_REGION_METADATA, bytes) };
        *dirty = true;
    }

    pub(crate) fn is_dirty(&self) -> bool {
        *self.dirty.lock()
    }

    pub(crate) fn flush(&self) -> Result<bool> {
        let mut dirty = self.dirty.lock();
        if !*dirty {
            return Ok(false);
        }
        if self.mmap.len() != 0 {
            self.mmap.flush_async()?;
        }
        self.file.sync_all()?;
        *dirty = false;
        Ok(true)
    }
}
