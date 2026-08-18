use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    os::fd::AsRawFd,
};

use parking_lot::RwLockReadGuard;
use rawdb::{Region, RegionMetadata};

use crate::{AnyStoredVec, BUFFER_SIZE, HEADER_OFFSET, VecIndex, VecValue, likely};

use super::super::{RawStrategy, ReadWriteRawVec};

/// Buffer size aligned to SIZE_OF_T for raw I/O reads.
const fn aligned_buffer_size<T>() -> usize {
    let size_of_t = size_of::<T>();
    (BUFFER_SIZE / size_of_t) * size_of_t
}

/// Buffered file I/O source for reading stored data sequentially.
///
/// Better than mmap for nonresident sequential scans. Uses a dedicated file
/// handle with OS readahead. Only sees stored (persisted) values.
pub struct RawIoSource<'a, I, T, S> {
    file: File,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
    file_offset: usize,
    end_offset: usize,
    _lock: RwLockReadGuard<'a, RegionMetadata>,
    _marker: PhantomData<(I, T, S)>,
}

impl<'a, I, T, S> RawIoSource<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const NORMAL_BUFFER_SIZE: usize = aligned_buffer_size::<T>();

    pub(crate) fn new(vec: &'a ReadWriteRawVec<I, T, S>, from: usize, to: usize) -> Self {
        Self::new_from_parts(vec.region(), vec.stored_len(), from, to)
    }

    pub(crate) fn new_from_parts(
        region: &'a Region,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let file = region.open_db_read_only_file().expect("open file");
        let region_meta = region.meta();
        let region_start = region_meta.start();
        let start_offset = region_start + HEADER_OFFSET;
        let from = from.min(stored_len);
        let to = to.min(stored_len);

        let from_offset = start_offset + from * Self::SIZE_OF_T;
        let end_offset = start_offset + to * Self::SIZE_OF_T;

        let mut this = Self {
            file,
            buffer: Vec::new(),
            buffer_pos: 0,
            buffer_len: 0,
            file_offset: from_offset,
            end_offset,
            _lock: region_meta,
            _marker: PhantomData,
        };

        if likely(this.can_read_file()) {
            this.file
                .seek(SeekFrom::Start(from_offset as u64))
                .expect("Failed to seek");
        }

        this
    }

    #[inline(always)]
    fn can_read_file(&self) -> bool {
        self.file_offset < self.end_offset
    }

    #[inline(always)]
    fn cant_read_file(&self) -> bool {
        self.file_offset >= self.end_offset
    }

    #[inline(always)]
    fn remaining_file_bytes(&self) -> usize {
        self.end_offset - self.file_offset
    }

    #[inline(always)]
    fn refill_buffer(&mut self) {
        if self.buffer.is_empty() {
            self.buffer.resize(Self::NORMAL_BUFFER_SIZE, 0);
        }
        let buffer_len = self.remaining_file_bytes().min(Self::NORMAL_BUFFER_SIZE);
        self.file
            .read_exact(&mut self.buffer[..buffer_len])
            .expect("Failed to read file buffer");
        self.file_offset += buffer_len;
        self.buffer_len = buffer_len;
        self.buffer_pos = 0;
    }

    /// Reads native-layout values directly into the destination allocation.
    pub(crate) fn read_into(self, output: &mut Vec<T>) {
        debug_assert!(S::IS_NATIVE_LAYOUT);
        let bytes = self.remaining_file_bytes();
        debug_assert!(bytes.is_multiple_of(Self::SIZE_OF_T));
        let values = bytes / Self::SIZE_OF_T;
        output.reserve(values);
        let old_len = output.len();
        let destination = output.spare_capacity_mut().as_mut_ptr().cast::<u8>();
        let mut read = 0;

        while read < bytes {
            let read_len = (bytes - read).min(i32::MAX as usize);
            let result = unsafe {
                libc::read(
                    self.file.as_raw_fd(),
                    destination.add(read).cast(),
                    read_len,
                )
            };
            if result > 0 {
                read += result as usize;
                continue;
            }
            if result == 0 {
                panic!("unexpected end of raw vector");
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                panic!("failed to read raw vector: {error}");
            }
        }

        unsafe { output.set_len(old_len + values) };
    }

    /// Fold all remaining elements — own implementation so LLVM can vectorize the inner loop.
    #[inline(always)]
    pub(crate) fn fold<B, F: FnMut(B, T) -> B>(mut self, init: B, mut f: F) -> B {
        let mut accum = init;
        loop {
            let ptr = self.buffer.as_ptr();
            let mut pos = self.buffer_pos;
            let end = self.buffer_len;
            while pos + Self::SIZE_OF_T <= end {
                accum = f(accum, unsafe { S::read_from_ptr(ptr, pos) });
                pos += Self::SIZE_OF_T;
            }
            self.buffer_pos = end;

            if self.cant_read_file() {
                break;
            }
            self.refill_buffer();
        }
        accum
    }

    /// Fallible fold with early exit on error.
    #[inline(always)]
    pub(crate) fn try_fold<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        mut self,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let mut accum = init;
        loop {
            let ptr = self.buffer.as_ptr();
            let mut pos = self.buffer_pos;
            let end = self.buffer_len;
            while pos + Self::SIZE_OF_T <= end {
                accum = f(accum, unsafe { S::read_from_ptr(ptr, pos) })?;
                pos += Self::SIZE_OF_T;
            }
            self.buffer_pos = end;

            if self.cant_read_file() {
                break;
            }
            self.refill_buffer();
        }
        Ok(accum)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{AnyStoredVec, BytesVec, Database, ImportableVec, Version, WritableVec};

    use super::RawIoSource;

    #[test]
    fn read_into_appends_the_requested_range() {
        let temp = tempdir().unwrap();
        let db = Database::open(temp.path()).unwrap();
        let mut vec: BytesVec<usize, u64> =
            BytesVec::forced_import(&db, "values", Version::ONE).unwrap();
        let values: Vec<_> = (0..10_000).map(|index| index as u64 * 37).collect();
        for &value in &values {
            vec.push(value);
        }
        vec.write().unwrap();

        let mut output = vec![u64::MAX];
        RawIoSource::new(&vec, 117, 9_731).read_into(&mut output);
        assert_eq!(&output[1..], &values[117..9_731]);
    }
}
