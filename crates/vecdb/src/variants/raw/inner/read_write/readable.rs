use rawdb::unlikely;

use crate::{AnyStoredVec, HEADER_OFFSET, RawIoSource, ReadableVec, VecIndex, VecValue};

use super::{super::RawStrategy, ReadWriteRawVec};

impl<I, T, S> ReadableVec<I, T> for ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        let len = self.base.len();
        if index >= len {
            return None;
        }

        if unlikely(!self.holes().is_empty()) && self.holes().contains(&index) {
            return None;
        }

        let stored_len = self.stored_len();
        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }

        if unlikely(!self.updated().is_empty())
            && let Some(value) = self.updated().get(&index)
        {
            return Some(value.clone());
        }

        Some(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * size_of::<T>())
        }))
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        buf.reserve(to - from);

        if self.has_dirty_stored() {
            self.fold_dirty(from, to, (), |(), v| buf.push(v));
            return;
        }

        let stored_len = self.stored_len();

        if from < stored_len {
            let stored_to = to.min(stored_len);
            if S::IS_NATIVE_LAYOUT {
                let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
                let bytes = (stored_to - from) * Self::SIZE_OF_T;
                if self.region().prefers_mmap(offset, bytes) {
                    let reader = self.raw_reader();
                    let src = unsafe {
                        std::slice::from_raw_parts(
                            reader
                                .prefixed(HEADER_OFFSET)
                                .as_ptr()
                                .add(from * Self::SIZE_OF_T)
                                as *const T,
                            stored_to - from,
                        )
                    };
                    buf.extend_from_slice(src);
                } else {
                    RawIoSource::new(self, from, stored_to).read_into(buf);
                }
            } else {
                self.fold_source(from, stored_to, (), |(), v| buf.push(v));
            }
        }

        if to > stored_len {
            let push_from = from.max(stored_len);
            let pushed = self.base.pushed();
            let start = push_from - stored_len;
            let end = (to - stored_len).min(pushed.len());
            buf.extend_from_slice(&pushed[start..end]);
        }
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), v| f(v));
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        Self: Sized,
    {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }

        if self.has_dirty_stored() {
            return self.fold_dirty(from, to, init, f);
        }

        let stored_len = self.stored_len();

        if to <= stored_len {
            return self.fold_source(from, to, init, f);
        }

        let mut acc = init;
        if from < stored_len {
            acc = self.fold_source(from, stored_len, acc, &mut f);
        }
        self.base.fold_pushed(from, to, acc, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E>
    where
        Self: Sized,
    {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }

        if self.has_dirty_stored() {
            return self.try_fold_dirty(from, to, init, f);
        }

        let stored_len = self.stored_len();

        if to <= stored_len {
            return self.try_fold_source(from, to, init, f);
        }

        let mut acc = init;
        if from < stored_len {
            acc = self.try_fold_source(from, stored_len, acc, &mut f)?;
        }
        self.base.try_fold_pushed(from, to, acc, f)
    }
}
