use crate::{HEADER_OFFSET, RawIoSource, ReadableVec, VecIndex, VecValue};

use super::{super::RawStrategy, ReadOnlyRawVec};

impl<I, T, S> ReadableVec<I, T> for ReadOnlyRawVec<I, T, S>
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
        if S::IS_NATIVE_LAYOUT {
            let offset = HEADER_OFFSET + from * size_of::<T>();
            let bytes = (to - from) * size_of::<T>();
            if self.base.region().prefers_mmap(offset, bytes) {
                let reader = self.base.region().create_reader();
                let src = unsafe {
                    std::slice::from_raw_parts(
                        reader
                            .prefixed(HEADER_OFFSET)
                            .as_ptr()
                            .add(from * size_of::<T>()) as *const T,
                        to - from,
                    )
                };
                buf.extend_from_slice(src);
            } else {
                RawIoSource::<I, T, S>::new_from_parts(self.base.region(), len, from, to)
                    .read_into(buf);
            }
        } else {
            self.fold_source(from, to, len, (), |(), v| buf.push(v));
        }
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), v| f(v));
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B
    where
        Self: Sized,
    {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }
        self.fold_source(from, to, len, init, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
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
        self.try_fold_source(from, to, len, init, f)
    }
}
