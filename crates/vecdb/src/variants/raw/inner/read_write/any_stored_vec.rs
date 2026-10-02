use std::{mem, path::PathBuf, slice};

use rawdb::{Database, Region};

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, AnyVec, Error, HEADER_OFFSET, Header, Result, Stamp, VecIndex, VecValue,
    WritableVec, cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> AnyStoredVec for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
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
        (self.region().meta().byte_len() - HEADER_OFFSET) / Self::SIZE_OF_T
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.base.stored_len()
    }

    fn write(&mut self) -> Result<bool> {
        let guard = self.header().begin_write()?;
        let result = self.with_cache_update(self.stored_len(), |this| {
            let stored_len = this.stored_len();
            let pushed_len = this.base.pushed().len();
            let real_stored_len = this.real_stored_len();
            let truncated = stored_len < real_stored_len;
            let has_new_data = pushed_len != 0;

            if stored_len > real_stored_len {
                return Err(Error::CorruptedRegion {
                    name: this.name().to_string(),
                    region_len: real_stored_len,
                });
            }

            if !truncated && !has_new_data {
                return this.base.write_header_if_needed();
            }

            let from = stored_len * Self::SIZE_OF_T + HEADER_OFFSET;

            if has_new_data {
                // Take the pushed buffer to free its heap allocation after writing.
                let taken = mem::take(this.base.mut_pushed());
                if S::IS_NATIVE_LAYOUT {
                    // Bulk write: memory layout matches serialized format, skip per-value
                    // serialization entirely. Single memcpy from pushed buffer to mmap.
                    let bytes = unsafe {
                        slice::from_raw_parts(
                            taken.as_ptr() as *const u8,
                            taken.len() * Self::SIZE_OF_T,
                        )
                    };
                    this.region().truncate_write(from, bytes)?;
                } else {
                    let mut bytes = Vec::with_capacity(pushed_len * Self::SIZE_OF_T);
                    for v in &taken {
                        S::write_to_vec(v, &mut bytes);
                    }
                    this.region().truncate_write(from, &bytes)?;
                }
                this.base.write_header_if_needed()?;
                this.base.update_stored_len(stored_len + pushed_len);
                if let Some(cache) = C::cache(&this.cache) {
                    cache.extend_tail(stored_len, &taken);
                }
            } else {
                this.region().truncate(from)?;
                this.base.write_header_if_needed()?;
            }
            Ok(true)
        });
        guard.finish(result)
    }

    fn region(&self) -> &Region {
        self.base.region()
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        self.base
            .serialize_changes::<S>(|from, to| self.collect_stored_range(from, to))
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        <Self as WritableVec<I, T>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<I, T>>::save_rollback_state(self)
    }

    fn remove(self) -> Result<()> {
        Self::remove(self)
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        <Self as WritableVec<I, T>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> Result<()> {
        <Self as WritableVec<I, T>>::reset(self)
    }
}
