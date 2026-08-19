use std::{mem, path::PathBuf};

use log::debug;
use rawdb::{Database, Region, unlikely};

use crate::{AnyStoredVec, Bytes, HEADER_OFFSET, Header, Stamp, WritableVec};

use super::{super::RawStrategy, ReadWriteRawVec};

impl<I, T, S> AnyStoredVec for ReadWriteRawVec<I, T, S>
where
    I: crate::VecIndex,
    T: crate::VecValue,
    S: RawStrategy<T>,
{
    #[inline]
    fn db_path(&self) -> PathBuf {
        self.base.db_path()
    }

    #[inline]
    fn header(&self) -> &Header {
        self.base.header()
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        self.base.mut_header()
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.base.saved_stamped_changes()
    }

    fn db(&self) -> Database {
        self.region().db()
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        (self.region().meta().len() - HEADER_OFFSET) / Self::SIZE_OF_T
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.base.stored_len()
    }

    fn write(&mut self) -> crate::Result<bool> {
        self.base.write_header_if_needed()?;

        let stored_len = self.stored_len();
        let pushed_len = self.base.pushed().len();
        let real_stored_len = self.real_stored_len();
        // After rollback, stored_len can be > real_stored_len (missing items are in updated map)
        let truncated = stored_len < real_stored_len;
        let expanded = stored_len > real_stored_len;
        let has_new_data = pushed_len != 0;
        let has_updated_data = !self.updated().is_empty();
        let has_holes = !self.holes().is_empty();
        let had_holes = self.has_stored_holes;

        if !truncated && !expanded && !has_new_data && !has_updated_data && !has_holes && !had_holes
        {
            return Ok(false);
        }

        let from = stored_len * Self::SIZE_OF_T + HEADER_OFFSET;

        if has_new_data {
            // Take the pushed buffer to free its heap allocation after writing.
            let taken = mem::take(self.base.mut_pushed());
            if S::IS_NATIVE_LAYOUT {
                // Bulk write: memory layout matches serialized format, skip per-value
                // serialization entirely. Single memcpy from pushed buffer to mmap.
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        taken.as_ptr() as *const u8,
                        taken.len() * Self::SIZE_OF_T,
                    )
                };
                self.region().truncate_write(from, bytes)?;
            } else {
                let mut bytes = Vec::with_capacity(pushed_len * Self::SIZE_OF_T);
                for v in &taken {
                    S::write_to_vec(v, &mut bytes);
                }
                self.region().truncate_write(from, &bytes)?;
            }
            self.base.update_stored_len(stored_len + pushed_len);
        } else if truncated {
            self.region().truncate(from)?;
        }

        if has_updated_data {
            let updated = self.updated.take_current();
            let region = self.region();

            if unlikely(expanded) {
                // After rollback, updates may extend beyond current disk length.
                // Use write_at which handles extension (slower but necessary).
                let mut bytes = Vec::with_capacity(Self::SIZE_OF_T);
                for (index, value) in updated {
                    let offset = index * Self::SIZE_OF_T + HEADER_OFFSET;
                    bytes.clear();
                    S::write_to_vec(&value, &mut bytes);
                    region.write_at(&bytes, offset)?;
                }
            } else {
                // Normal case: write directly to mmap, no intermediate allocations
                region.batch_write_ordered(
                    updated
                        .into_iter()
                        .map(|(index, value)| (index * Self::SIZE_OF_T + HEADER_OFFSET, value)),
                    Self::SIZE_OF_T,
                    S::write_to_slice,
                );
            }
        }

        if has_holes {
            self.has_stored_holes = true;
            let holes_region = self
                .region()
                .db()
                .create_region_if_needed(&self.holes_region_name())?;
            let holes = self.holes();
            let mut bytes = Vec::with_capacity(holes.len() * size_of::<usize>());
            for i in holes {
                bytes.extend(i.to_bytes());
            }
            holes_region.truncate_write(0, &bytes)?;
        } else if had_holes {
            self.has_stored_holes = false;
            let db = self.region().db();
            let holes_name = self.holes_region_name();
            debug!("{}: removing holes region '{}'", db, holes_name);
            db.remove_region(&holes_name)?;
        }

        Ok(true)
    }

    fn region(&self) -> &Region {
        self.base.region()
    }

    fn serialize_changes(&self) -> crate::Result<Vec<u8>> {
        self.serialize_raw_changes()
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<I, T>>::save_rollback_state(self)
    }

    fn remove(self) -> crate::Result<()> {
        Self::remove(self)
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::reset(self)
    }
}
