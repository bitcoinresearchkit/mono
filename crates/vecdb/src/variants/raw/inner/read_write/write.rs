use std::mem;

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, AnyVec, Error, HEADER_OFFSET, Result, VecIndex, VecValue, cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    /// Rows on disk.
    #[inline]
    pub(super) fn written_len(&self) -> usize {
        (self.region().meta().byte_len() - HEADER_OFFSET) / Self::SIZE_OF_T
    }

    /// Appends the pushed values after the stored length, cutting a pending truncation.
    pub(super) fn write_pushed(&mut self) -> Result<bool> {
        self.with_cache_update(self.stored_len(), |this| {
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
                let taken = mem::take(this.base.mut_pushed());
                this.region()
                    .truncate_write(from, &S::values_bytes(&taken))?;
                this.base.write_header_if_needed()?;
                this.base.update_stored_len(stored_len + pushed_len);
                if let Some(cache) = C::cache(&this.cache) {
                    cache.extend_tail(stored_len, &taken);
                }
                this.base.recycle_pushed(taken);
            } else {
                this.region().truncate(from)?;
                this.base.write_header_if_needed()?;
            }
            Ok(true)
        })
    }
}
