use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

use parking_lot::RwLock;
use rawdb::{Database, Region};

use crate::{
    AnyStoredVec, AnyVec, BytesVec, BytesVecReader, Error, Header, ImportOptions, ImportableVec,
    MutableVec, OverflowVecReader, OverflowVecValue, ReadOnlyOverflowVec, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, Result, SharedLen, Stamp, StoredVec, TypedVec, VecIndex,
    Version, WritableVec, short_type_name, unlikely,
};

use super::DECODE_CHUNK_SIZE;

const VERSION: Version = Version::ONE;

/// One logical raw vector with compact inline values and a full-width overflow
/// sidecar for the uncommon values that do not fit.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    compact: MutableVec<BytesVec<I, T::Compact>>,
    overflow: MutableVec<BytesVec<usize, T>>,
    pushed: Vec<T>,
    visible_len: SharedLen,
    gate: Arc<RwLock<()>>,
}

impl<I, T> OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn overflow_name(name: &str) -> String {
        format!("{name}_overflow")
    }

    fn import_inner(mut options: ImportOptions, forced: bool) -> Result<Self> {
        options.version = options.version.combine(VERSION).combine(T::VERSION);
        let overflow_name = Self::overflow_name(options.name);
        let overflow_options = ImportOptions {
            name: &overflow_name,
            initial_capacity: Some(0),
            ..options
        };

        let mut compact = if forced {
            MutableVec::<BytesVec<_, _>>::forced_import_with(options)?
        } else {
            MutableVec::<BytesVec<_, _>>::import_with(options)?
        };
        let mut overflow = if forced {
            MutableVec::<BytesVec<_, _>>::forced_import_with(overflow_options)?
        } else {
            MutableVec::<BytesVec<_, _>>::import_with(overflow_options)?
        };

        if compact.stamp() != overflow.stamp() {
            if !forced {
                return Err(Error::StampMismatch {
                    file: overflow.stamp(),
                    vec: compact.stamp(),
                });
            }
            compact.reset()?;
            overflow.reset()?;
            overflow.write()?;
            compact.write()?;
        }

        let len = compact.len();
        Ok(Self {
            compact,
            overflow,
            pushed: Vec::new(),
            visible_len: SharedLen::new(len),
            gate: Arc::new(RwLock::new(())),
        })
    }

    #[inline(always)]
    fn decode(&self, compact: T::Compact, reader: &OverflowVecReader<I, T>) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            reader
                .overflow(&self.overflow, overflow_index.unwrap())
                .expect("OverflowVec pointer must reference a stored value")
        } else {
            T::from_compact(compact)
        }
    }

    #[inline(always)]
    fn decode_current(&self, compact: T::Compact) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            self.overflow
                .collect_one_at(overflow_index.unwrap())
                .expect("OverflowVec pointer must reference a stored value")
        } else {
            T::from_compact(compact)
        }
    }

    #[inline(always)]
    fn decode_with_overflow_reader(
        compact: T::Compact,
        overflow_vec: &MutableVec<BytesVec<usize, T>>,
        overflow: &BytesVecReader<usize, T>,
    ) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            overflow_vec
                .get_with_reader_at(overflow_index.unwrap(), overflow)
                .expect("OverflowVec pointer must reference a stored value")
        } else {
            T::from_compact(compact)
        }
    }

    fn store_overflow(overflow: &mut MutableVec<BytesVec<usize, T>>, value: T) -> usize {
        overflow
            .fill_first_hole_or_push(value)
            .expect("OverflowVec sidecar hole must remain writable")
    }

    #[inline(always)]
    fn encode_with_overflow(
        overflow: &mut MutableVec<BytesVec<usize, T>>,
        value: &T,
    ) -> T::Compact {
        if let Some(compact) = value.to_compact() {
            debug_assert!(T::overflow_index(compact).is_none());
            return compact;
        }

        let index = Self::store_overflow(overflow, value.clone());
        let compact = T::from_overflow_index(index);
        debug_assert_eq!(T::overflow_index(compact), Some(index));
        compact
    }

    fn encode(&mut self, value: &T) -> T::Compact {
        Self::encode_with_overflow(&mut self.overflow, value)
    }

    fn fill_hole_at(&mut self, index: usize, value: T) -> Result<()> {
        debug_assert!(self.compact.holes().contains(&index));
        let compact = self.encode(&value);
        self.compact.update_at(index, compact)?;
        let stored_len = self.compact.stored_len();
        if index >= stored_len {
            self.pushed[index - stored_len] = value;
        }
        Ok(())
    }

    fn replace_compact(&mut self, old: T::Compact, value: &T) -> Result<T::Compact> {
        Ok(match (T::overflow_index(old), value.to_compact()) {
            (Some(overflow_index), None) => {
                self.overflow.update_at(overflow_index, value.clone())?;
                old
            }
            (Some(overflow_index), Some(compact)) => {
                self.overflow.delete_at(overflow_index);
                compact
            }
            (None, Some(compact)) => compact,
            (None, None) => {
                let overflow_index = Self::store_overflow(&mut self.overflow, value.clone());
                T::from_overflow_index(overflow_index)
            }
        })
    }

    fn update_at_with_reader(
        &mut self,
        index: usize,
        value: T,
        reader: &OverflowVecReader<I, T>,
    ) -> Result<()> {
        if self.compact.holes().contains(&index) {
            return self.fill_hole_at(index, value);
        }

        let old = reader
            .compact(&self.compact, I::from(index))
            .ok_or_else(|| Error::IndexTooHigh {
                index,
                len: self.len(),
                name: self.name().to_string(),
            })?;
        let compact = self.replace_compact(old, &value)?;

        self.compact.update_at(index, compact)?;
        let stored_len = self.compact.stored_len();
        if index >= stored_len {
            self.pushed[index - stored_len] = value;
        }
        Ok(())
    }

    fn update_at(&mut self, index: usize, value: T) -> Result<()> {
        let reader = self.reader();
        self.update_at_with_reader(index, value, &reader)
    }

    fn delete_with_reader(&mut self, index: I, reader: &OverflowVecReader<I, T>) {
        if let Some(compact) = reader.compact(&self.compact, index) {
            if let Some(overflow_index) = T::overflow_index(compact) {
                self.overflow.delete_at(overflow_index);
            }
            self.compact.delete(index);
        }
    }

    pub fn read_only_clone(&self) -> ReadOnlyOverflowVec<I, T> {
        ReadOnlyOverflowVec::new(
            self.compact.read_only_clone(),
            self.overflow.read_only_clone(),
            self.visible_len.clone(),
            Arc::clone(&self.gate),
        )
    }

    pub fn reader(&self) -> OverflowVecReader<I, T> {
        OverflowVecReader::new(&self.compact, &self.overflow)
    }

    #[inline(always)]
    pub fn get_with_reader(&self, index: I, reader: &OverflowVecReader<I, T>) -> Option<T> {
        reader
            .compact(&self.compact, index)
            .map(|compact| self.decode(compact, reader))
    }

    pub fn holes(&self) -> &BTreeSet<usize> {
        self.compact.holes()
    }

    pub fn reserve_pushed(&mut self, additional: usize) {
        self.compact.reserve_pushed(additional);
        self.pushed.reserve(additional);
    }

    #[inline(always)]
    pub fn push(&mut self, value: T) {
        let compact = self.encode(&value);
        self.compact.push(compact);
        self.pushed.push(value);
    }

    pub fn update(&mut self, index: I, value: T) -> Result<()> {
        self.update_at(index.to_usize(), value)
    }

    /// Replaces one final value per index, sorting and applying the owned batch.
    pub fn update_many(&mut self, mut values: Vec<(I, T)>) -> Result<()> {
        values.sort_unstable_by_key(|(index, _)| index.to_usize());
        debug_assert!(
            values.windows(2).all(|pair| pair[0].0 != pair[1].0),
            "OverflowVec update batch must contain one final value per index"
        );
        if let Some((index, _)) = values.last()
            && index.to_usize() >= self.len()
        {
            let index = index.to_usize();
            return Err(Error::IndexTooHigh {
                index,
                len: self.len(),
                name: self.name().to_string(),
            });
        }

        let reader = self.reader();
        let stored_len = self.compact.stored_len();
        let mut compact_updates = Vec::with_capacity(values.len());
        for (index, value) in values {
            let index_at = index.to_usize();
            let compact = if self.compact.holes().contains(&index_at) {
                self.encode(&value)
            } else {
                let old = reader
                    .compact(&self.compact, index)
                    .expect("validated OverflowVec update index");
                self.replace_compact(old, &value)?
            };

            compact_updates.push((index, compact));
            if index_at >= stored_len {
                self.pushed[index_at - stored_len] = value;
            }
        }
        self.compact.update_many(compact_updates)
    }

    pub fn delete(&mut self, index: I) {
        let reader = self.reader();
        self.delete_with_reader(index, &reader);
    }

    pub fn delete_many(&mut self, indices: impl IntoIterator<Item = I>) {
        let reader = self.reader();
        for index in indices {
            self.delete_with_reader(index, &reader);
        }
    }

    pub fn get_first_empty_index(&self) -> I {
        self.compact.get_first_empty_index()
    }

    pub fn fill_first_hole_or_push(&mut self, value: T) -> Result<I> {
        if !self.compact.holes().is_empty() {
            let compact = self.encode(&value);
            let stored_len = self.compact.stored_len();
            let index = self.compact.fill_first_hole_or_push(compact)?.to_usize();
            if index >= stored_len {
                self.pushed[index - stored_len] = value;
            }
            return Ok(I::from(index));
        }
        let index = I::from(self.len());
        self.push(value);
        Ok(index)
    }

    /// Fills the lowest available indexes, then appends, preserving input order.
    pub fn fill_holes_or_push_many(&mut self, values: Vec<T>) -> Vec<I> {
        let compacts = values
            .iter()
            .map(|value| Self::encode_with_overflow(&mut self.overflow, value));
        let indices = self.compact.fill_holes_or_push_many(compacts);
        let stored_len = self.compact.stored_len();
        self.pushed.reserve(
            self.compact
                .len()
                .saturating_sub(stored_len + self.pushed.len()),
        );
        for (&index, value) in indices.iter().zip(values) {
            let index = index.to_usize();
            if index >= stored_len {
                let offset = index - stored_len;
                if offset < self.pushed.len() {
                    self.pushed[offset] = value;
                } else {
                    debug_assert_eq!(offset, self.pushed.len());
                    self.pushed.push(value);
                }
            }
        }
        indices
    }

    fn write_inner(&mut self) -> Result<bool> {
        let overflow = self.overflow.write()?;
        let compact = self.compact.write()?;
        self.pushed.clear();
        self.visible_len.set(self.compact.stored_len());
        Ok(compact || overflow)
    }
}

impl<I, T> ImportableVec for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import_with((db, name, version).into())
    }

    fn import_with(options: ImportOptions) -> Result<Self> {
        Self::import_inner(options, false)
    }

    fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::forced_import_with((db, name, version).into())
    }

    fn forced_import_with(options: ImportOptions) -> Result<Self> {
        Self::import_inner(options, true)
    }
}

impl<I, T> AnyVec for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn version(&self) -> Version {
        self.compact.version()
    }

    fn name(&self) -> &str {
        self.compact.name()
    }

    fn len(&self) -> usize {
        self.compact.len()
    }

    fn is_mutable(&self) -> bool {
        true
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }

    fn region_names(&self) -> Vec<String> {
        let mut names = self.compact.region_names();
        names.extend(self.overflow.region_names());
        names
    }
}

impl<I, T> TypedVec for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    type I = I;
    type T = T;
}

impl<I, T> AnyStoredVec for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn db_path(&self) -> PathBuf {
        self.compact.db_path()
    }

    fn region(&self) -> &Region {
        self.compact.region()
    }

    fn header(&self) -> &Header {
        self.compact.header()
    }

    fn mut_header(&mut self) -> &mut Header {
        self.compact.mut_header()
    }

    fn saved_stamped_changes(&self) -> u16 {
        self.compact.saved_stamped_changes()
    }

    fn write(&mut self) -> Result<bool> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.write_inner()
    }

    fn flush(&mut self) -> Result<()> {
        if self.write()? {
            self.overflow.region().flush()?;
            self.compact.region().flush()?;
        }
        Ok(())
    }

    fn db(&self) -> Database {
        self.compact.db()
    }

    fn real_stored_len(&self) -> usize {
        self.compact.real_stored_len()
    }

    fn stored_len(&self) -> usize {
        self.compact.stored_len()
    }

    fn update_stamp(&mut self, stamp: Stamp) {
        self.overflow.update_stamp(stamp);
        self.compact.update_stamp(stamp);
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        self.stamped_write_with_changes(stamp)
    }

    fn any_save_rollback_state(&mut self) {
        self.save_rollback_state();
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        let compact = self.compact.serialize_changes()?;
        let overflow = self.overflow.serialize_changes()?;
        let mut changes = Vec::with_capacity(8 + compact.len() + overflow.len());
        changes.extend_from_slice(&(compact.len() as u64).to_le_bytes());
        changes.extend(compact);
        changes.extend(overflow);
        Ok(changes)
    }

    fn remove(self) -> Result<()> {
        self.overflow.remove()?;
        self.compact.remove()
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        self.truncate_if_needed_at(index)
    }

    fn any_reset(&mut self) -> Result<()> {
        self.reset()
    }
}

impl<I, T> WritableVec<I, T> for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn push(&mut self, value: T) {
        OverflowVec::push(self, value);
    }

    fn pushed(&self) -> &[T] {
        &self.pushed
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        let len = self.len();
        if index >= len {
            return Ok(());
        }

        let reader = self.reader();
        for current in index..len {
            if let Some(compact) = reader.compact(&self.compact, I::from(current))
                && let Some(overflow_index) = T::overflow_index(compact)
            {
                self.overflow.delete_at(overflow_index);
            }
        }

        self.compact.truncate_if_needed_at(index)?;
        self.pushed.truncate(self.compact.pushed().len());
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.overflow.reset()?;
        self.compact.reset()?;
        self.pushed.clear();
        self.visible_len.set(0);
        Ok(())
    }

    fn reset_unsaved(&mut self) {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.overflow.reset_unsaved();
        self.compact.reset_unsaved();
        self.pushed.clear();
        self.visible_len.set(self.compact.stored_len());
    }

    fn is_dirty(&self) -> bool {
        self.compact.is_dirty() || self.overflow.is_dirty()
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.overflow.stamped_write_with_changes(stamp)?;
        self.compact.stamped_write_with_changes(stamp)?;
        self.pushed.clear();
        self.visible_len.set(self.compact.stored_len());
        Ok(())
    }

    fn rollback(&mut self) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.overflow.rollback()?;
        self.compact.rollback()?;
        self.pushed.clear();
        self.visible_len.set(self.compact.stored_len());
        Ok(())
    }

    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>> {
        let compact = self.compact.find_rollback_files()?;
        let overflow = self.overflow.find_rollback_files()?;
        debug_assert!(compact.keys().eq(overflow.keys()));
        Ok(compact)
    }

    fn save_rollback_state(&mut self) {
        self.overflow.save_rollback_state();
        self.compact.save_rollback_state();
    }
}

impl<I, T> ReadableVec<I, T> for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        self.compact
            .collect_one_at(index)
            .map(|compact| self.decode_current(compact))
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let len = self.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        let overflow_vec = &self.overflow;
        let overflow = overflow_vec.reader();
        buf.reserve(to - from);
        if to - from > DECODE_CHUNK_SIZE {
            let mut compact = Vec::with_capacity(DECODE_CHUNK_SIZE);
            for chunk_from in (from..to).step_by(DECODE_CHUNK_SIZE) {
                compact.clear();
                self.compact.read_into_at(
                    chunk_from,
                    (chunk_from + DECODE_CHUNK_SIZE).min(to),
                    &mut compact,
                );
                buf.extend(compact.iter().copied().map(|value| {
                    Self::decode_with_overflow_reader(value, overflow_vec, &overflow)
                }));
            }
            return;
        }

        self.compact.for_each_range_at(from, to, |compact| {
            buf.push(Self::decode_with_overflow_reader(
                compact,
                overflow_vec,
                &overflow,
            ));
        });
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        let overflow_vec = &self.overflow;
        let overflow = overflow_vec.reader();
        self.compact
            .for_each_range_dyn_at(from, to, &mut |compact| {
                f(Self::decode_with_overflow_reader(
                    compact,
                    overflow_vec,
                    &overflow,
                ))
            });
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let overflow_vec = &self.overflow;
        let overflow = overflow_vec.reader();
        self.compact.fold_range_at(from, to, init, |acc, compact| {
            f(
                acc,
                Self::decode_with_overflow_reader(compact, overflow_vec, &overflow),
            )
        })
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let overflow_vec = &self.overflow;
        let overflow = overflow_vec.reader();
        self.compact
            .try_fold_range_at(from, to, init, |acc, compact| {
                f(
                    acc,
                    Self::decode_with_overflow_reader(compact, overflow_vec, &overflow),
                )
            })
    }
}

impl<I, T> StoredVec for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    type ReadOnly = ReadOnlyOverflowVec<I, T>;

    fn read_only_clone(&self) -> Self::ReadOnly {
        OverflowVec::read_only_clone(self)
    }
}

impl<I, T> ReadableCloneableVec<I, T> for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T> {
        ReadableBoxedVec::new(self.read_only_clone())
    }
}
