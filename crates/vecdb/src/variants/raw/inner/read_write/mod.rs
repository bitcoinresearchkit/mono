use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use log::info;
use rawdb::{Reader, unlikely};

mod any_stored_vec;
mod any_vec;
mod change;
mod readable;
mod rollback;
mod typed;
mod writable;

use crate::{
    AnyStoredVec, AnyVec, Bytes, Error, Format, HEADER_OFFSET, ImportOptions, MMAP_CROSSOVER_BYTES,
    RawIoSource, RawMmapSource, ReadWriteBaseVec, Result, VecIndex, VecReader, VecValue, Version,
    WithPrev, vec_region_name_with,
};

use super::{RawStrategy, ReadOnlyRawVec};

const VERSION: Version = Version::ONE;

/// Core implementation for raw storage vectors shared by BytesVec and ZeroCopyVec.
///
/// Parameterized by serialization strategy `S` to support different serialization approaches:
/// - `BytesStrategy`: Explicit little-endian serialization (portable)
/// - `ZeroCopyStrategy`: Native byte order via zerocopy (fast but not portable)
///
/// Provides holes (deleted indices) and updated values tracking for both vec types.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ReadWriteRawVec<I, T, S> {
    pub(crate) base: ReadWriteBaseVec<I, T>,
    pub(super) holes: WithPrev<BTreeSet<usize>>,
    pub(super) updated: WithPrev<BTreeMap<usize, T>>,
    pub(super) has_stored_holes: bool,
    _strategy: PhantomData<S>,
}

impl<I, T, S> ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub const SIZE_OF_T: usize = size_of::<T>();

    pub fn read_only_clone(&self) -> ReadOnlyRawVec<I, T, S> {
        ReadOnlyRawVec::new(self.base.read_only_base())
    }

    /// # Warning
    ///
    /// This will DELETE all existing data on format/version errors. Use with caution.
    pub fn forced_import_with(options: ImportOptions, format: Format) -> Result<Self> {
        let res = Self::import_with(options, format);
        match res {
            Err(Error::WrongEndian)
            | Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentVersion { .. }) => {
                info!("Resetting {}...", options.name);
                options
                    .db
                    .remove_region_if_exists(&Self::holes_region_name_with(options.name))?;
                options
                    .db
                    .remove_region_if_exists(&vec_region_name_with::<I>(options.name))?;
                Self::import_with(options, format)
            }
            _ => res,
        }
    }

    pub fn import_with(mut options: ImportOptions, format: Format) -> Result<Self> {
        options.version = options.version + VERSION;

        let db = options.db;
        let name = options.name;

        let base = ReadWriteBaseVec::import(options, format)?;

        // Raw format requires data to be aligned to SIZE_OF_T
        let region_len = base.region().meta().len();
        if region_len > HEADER_OFFSET
            && !(region_len - HEADER_OFFSET).is_multiple_of(Self::SIZE_OF_T)
        {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len,
            });
        }

        let holes = db
            .get_region(&Self::holes_region_name_with(name))
            .map(|region| {
                region
                    .create_reader()
                    .read_all()
                    .chunks(size_of::<usize>())
                    .map(usize::from_bytes)
                    .collect::<Result<BTreeSet<usize>>>()
            })
            .transpose()?;

        let mut this = Self {
            base,
            has_stored_holes: holes.is_some(),
            holes: WithPrev::new(holes.unwrap_or_default()),
            updated: WithPrev::default(),
            _strategy: PhantomData,
        };

        let len = this.real_stored_len();
        *this.base.mut_prev_stored_len() = len;
        this.base.update_stored_len(len);

        Ok(this)
    }

    pub fn remove(self) -> Result<()> {
        let db = self.base.db();
        let holes_region_name = self.holes_region_name();
        let has_stored_holes = self.has_stored_holes;

        self.base.remove()?;

        if has_stored_holes {
            db.remove_region(&holes_region_name)?;
        }

        Ok(())
    }

    pub(super) fn holes_region_name(&self) -> String {
        Self::holes_region_name_with(self.name())
    }

    fn holes_region_name_with(name: &str) -> String {
        format!("{}_holes", vec_region_name_with::<I>(name))
    }

    #[inline(always)]
    pub fn holes(&self) -> &BTreeSet<usize> {
        self.holes.current()
    }

    #[inline(always)]
    pub fn prev_holes(&self) -> &BTreeSet<usize> {
        self.holes.previous()
    }

    #[inline(always)]
    pub fn mut_holes(&mut self) -> &mut BTreeSet<usize> {
        self.holes.current_mut()
    }

    #[inline(always)]
    pub fn updated(&self) -> &BTreeMap<usize, T> {
        self.updated.current()
    }

    #[inline(always)]
    pub fn mut_updated(&mut self) -> &mut BTreeMap<usize, T> {
        self.updated.current_mut()
    }

    #[inline(always)]
    pub fn prev_updated(&self) -> &BTreeMap<usize, T> {
        self.updated.previous()
    }

    #[inline(always)]
    pub fn pushed(&self) -> &[T] {
        self.base.pushed()
    }

    #[inline(always)]
    pub fn mut_pushed(&mut self) -> &mut Vec<T> {
        self.base.mut_pushed()
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.base.mut_pushed().push(value);
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.base.reserve_pushed(additional);
    }

    #[inline]
    fn raw_reader(&self) -> Reader {
        self.base.region().create_reader()
    }

    #[inline]
    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_write(self)
    }

    #[inline]
    pub fn index_to_name(&self) -> String {
        self.base.index_to_name()
    }

    #[inline(always)]
    fn unchecked_read_at(&self, index: usize, reader: &Reader) -> T {
        let ptr = reader.prefixed(HEADER_OFFSET).as_ptr();
        unsafe { S::read_from_ptr(ptr, index * Self::SIZE_OF_T) }
    }

    #[inline]
    pub fn read_at_once(&self, index: usize) -> Result<T> {
        let len = self.stored_len();
        if index >= len {
            return Err(Error::IndexTooHigh {
                index,
                len,
                name: self.name().to_string(),
            });
        }

        Ok(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * Self::SIZE_OF_T)
        }))
    }

    #[inline]
    pub fn read_once(&self, index: I) -> Result<T> {
        self.read_at_once(index.to_usize())
    }

    /// Reads from an append-only vector's persisted or pushed layer.
    ///
    /// Use [`Self::get_with_reader`] when the vector may contain holes or updates.
    #[inline(always)]
    pub fn get_append_only(&self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        self.get_append_only_at(index.to_usize(), reader)
    }

    /// Raw-index form of [`Self::get_append_only`].
    #[inline(always)]
    pub fn get_append_only_at(&self, index: usize, reader: &VecReader<I, T, S>) -> Option<T> {
        debug_assert!(
            self.holes().is_empty() && self.updated().is_empty(),
            "get_append_only requires a vector without holes or updates"
        );
        // The reader snapshots the persisted boundary when it is created.
        // Using that boundary avoids reloading SharedLen for every lookup and
        // lets the inlined reader.get_at() reuse the same bounds check.
        let stored_len = reader.len();
        debug_assert_eq!(stored_len, self.stored_len(), "stale VecReader");
        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }
        Some(reader.get_at(index))
    }

    /// Reads the current value across holes, updates, persisted data, and pushes.
    #[inline]
    pub fn get_with_reader(&self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        self.get_with_reader_at(index.to_usize(), reader)
    }

    /// Raw-index form of [`Self::get_with_reader`].
    #[inline]
    pub fn get_with_reader_at(&self, index: usize, reader: &VecReader<I, T, S>) -> Option<T> {
        if unlikely(!self.holes().is_empty()) && self.holes().contains(&index) {
            return None;
        }

        let stored_len = reader.len();
        debug_assert_eq!(stored_len, self.stored_len(), "stale VecReader");

        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }

        if unlikely(!self.updated().is_empty())
            && let Some(updated_value) = self.updated().get(&index)
        {
            return Some(updated_value.clone());
        }

        Some(reader.get_at(index))
    }

    pub fn collect_holed(&self) -> Vec<Option<T>> {
        self.collect_holed_range(0, self.len())
    }

    pub fn collect_holed_range(&self, from: usize, to: usize) -> Vec<Option<T>> {
        let len = self.len();
        let from = from.min(len);
        let to = to.min(len);

        if from >= to {
            return vec![];
        }

        let reader = self.reader();

        (from..to)
            .map(|i| self.get_with_reader_at(i, &reader))
            .collect()
    }

    #[inline]
    pub fn update(&mut self, index: I, value: T) -> Result<()> {
        self.update_at(index.to_usize(), value)
    }

    #[inline]
    pub fn update_at(&mut self, index: usize, value: T) -> Result<()> {
        let stored_len = self.stored_len();

        if index >= stored_len {
            let Some(slot) = self.base.mut_pushed().get_mut(index - stored_len) else {
                return Err(Error::IndexTooHigh {
                    index,
                    len: stored_len,
                    name: self.name().to_string(),
                });
            };
            *slot = value;
            return Ok(());
        }

        if !self.holes().is_empty() {
            self.mut_holes().remove(&index);
        }

        self.mut_updated().insert(index, value);

        Ok(())
    }

    #[inline]
    pub fn delete(&mut self, index: I) {
        self.delete_at(index.to_usize())
    }

    #[inline]
    pub fn delete_at(&mut self, index: usize) {
        if index < self.len() {
            self.unchecked_delete_at(index);
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn unchecked_delete(&mut self, index: I) {
        self.unchecked_delete_at(index.to_usize())
    }

    #[inline]
    #[doc(hidden)]
    pub fn unchecked_delete_at(&mut self, index: usize) {
        if !self.updated().is_empty() {
            self.mut_updated().remove(&index);
        }
        self.mut_holes().insert(index);
    }

    #[inline]
    pub fn get_first_empty_index(&self) -> I {
        self.holes()
            .first()
            .copied()
            .unwrap_or_else(|| self.base.len())
            .into()
    }

    #[inline]
    pub fn fill_first_hole_or_push(&mut self, value: T) -> Result<I> {
        if let Some(hole) = self.mut_holes().pop_first().map(I::from) {
            self.update(hole, value)?;
            return Ok(hole);
        }
        self.base.mut_pushed().push(value);
        Ok(I::from(self.len() - 1))
    }

    pub fn take(&mut self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        self.take_at(index.to_usize(), reader)
    }

    pub fn take_at(&mut self, index: usize, reader: &VecReader<I, T, S>) -> Option<T> {
        let opt = self.get_with_reader_at(index, reader);
        if opt.is_some() {
            self.unchecked_delete_at(index);
        }
        opt
    }

    pub(crate) fn collect_stored_range(&self, from: usize, to: usize) -> Result<Vec<T>> {
        let reader = self.raw_reader();
        Ok((from..to)
            .map(|i| {
                if let Some(val) = self.prev_updated().get(&i) {
                    val.clone()
                } else {
                    self.unchecked_read_at(i, &reader)
                }
            })
            .collect())
    }

    #[inline]
    pub(super) fn has_dirty_stored(&self) -> bool {
        !self.holes().is_empty() || !self.updated().is_empty()
    }

    pub(super) fn truncate_dirty_at(&mut self, index: usize) {
        if self.holes().last().is_some_and(|&h| h >= index) {
            self.mut_holes().split_off(&index);
        }
        if self
            .updated()
            .last_key_value()
            .is_some_and(|(&k, _)| k >= index)
        {
            self.mut_updated().split_off(&index);
        }
    }

    #[inline(always)]
    pub(super) fn fold_source<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let range_bytes = (to - from) * Self::SIZE_OF_T;
        if range_bytes > MMAP_CROSSOVER_BYTES {
            RawIoSource::new(self, from, to).fold(init, f)
        } else {
            RawMmapSource::new(self, from, to).fold(init, f)
        }
    }

    #[inline(always)]
    pub(super) fn try_fold_source<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        let range_bytes = (to - from) * Self::SIZE_OF_T;
        if range_bytes > MMAP_CROSSOVER_BYTES {
            RawIoSource::new(self, from, to).try_fold(init, f)
        } else {
            RawMmapSource::new(self, from, to).try_fold(init, f)
        }
    }

    /// Own implementation (not delegating to try_fold_dirty) so LLVM can vectorize without `?` penalty.
    pub(super) fn fold_dirty<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let reader = self.raw_reader();
        let data_ptr = reader.prefixed(HEADER_OFFSET).as_ptr();
        let mut acc = init;

        let stored_to = to.min(stored_len);
        let mut hole_iter = self.holes().range(from..to).peekable();
        let mut update_iter = self.updated().range(from..stored_to).peekable();

        let mut byte_off = from * Self::SIZE_OF_T;
        for i in from..stored_to {
            if unlikely(hole_iter.peek() == Some(&&i)) {
                hole_iter.next();
                byte_off += Self::SIZE_OF_T;
                continue;
            }
            let val = if unlikely(update_iter.peek().is_some_and(|&(&k, _)| k == i)) {
                update_iter.next().unwrap().1.clone()
            } else {
                unsafe { S::read_from_ptr(data_ptr, byte_off) }
            };
            byte_off += Self::SIZE_OF_T;
            acc = f(acc, val);
        }

        let push_from = from.max(stored_len);
        if push_from < to {
            let pushed = self.base.pushed();
            for i in push_from..to {
                if unlikely(hole_iter.peek() == Some(&&i)) {
                    hole_iter.next();
                    continue;
                }
                if let Some(v) = pushed.get(i - stored_len) {
                    acc = f(acc, v.clone());
                }
            }
        }

        acc
    }

    pub(super) fn try_fold_dirty<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let stored_len = self.stored_len();
        let reader = self.raw_reader();
        let data_ptr = reader.prefixed(HEADER_OFFSET).as_ptr();
        let mut acc = init;

        let stored_to = to.min(stored_len);
        let mut hole_iter = self.holes().range(from..to).peekable();
        let mut update_iter = self.updated().range(from..stored_to).peekable();

        let mut byte_off = from * Self::SIZE_OF_T;
        for i in from..stored_to {
            if unlikely(hole_iter.peek() == Some(&&i)) {
                hole_iter.next();
                byte_off += Self::SIZE_OF_T;
                continue;
            }
            let val = if unlikely(update_iter.peek().is_some_and(|&(&k, _)| k == i)) {
                update_iter.next().unwrap().1.clone()
            } else {
                // SAFETY: i < stored_len, reader holds mmap guard
                unsafe { S::read_from_ptr(data_ptr, byte_off) }
            };
            byte_off += Self::SIZE_OF_T;
            acc = f(acc, val)?;
        }

        let push_from = from.max(stored_len);
        if push_from < to {
            let pushed = self.base.pushed();
            for i in push_from..to {
                if unlikely(hole_iter.peek() == Some(&&i)) {
                    hole_iter.next();
                    continue;
                }
                if let Some(v) = pushed.get(i - stored_len) {
                    acc = f(acc, v.clone())?;
                }
            }
        }

        Ok(acc)
    }

    pub fn fold_stored_io<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        if from >= to {
            return init;
        }
        RawIoSource::new(self, from, to).fold(init, f)
    }

    pub fn fold_stored_mmap<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        if from >= to {
            return init;
        }
        RawMmapSource::new(self, from, to).fold(init, f)
    }
}
