use std::{mem, path::PathBuf};

use rawdb::{Database, Region};

use crate::{AnyStoredVec, AnyVec, Error, HEADER_OFFSET, Header, Stamp, WritableVec};

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
        let truncated = stored_len < real_stored_len;
        let has_new_data = pushed_len != 0;

        if stored_len > real_stored_len {
            return Err(Error::CorruptedRegion {
                name: self.name().to_string(),
                region_len: real_stored_len,
            });
        }

        if !truncated && !has_new_data {
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
