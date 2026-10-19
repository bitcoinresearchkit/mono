use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    ops::Deref,
    sync::Arc,
};

use parking_lot::RwLock;

use log::debug;

use crate::{
    AnyStoredVec, AnyVec, Bytes, Error, Result, TypedVec, VecIndex, WithPrev, vec_region_name_with,
};

pub mod any_stored_vec;
pub mod any_vec;
pub mod importable;
pub mod raw;
pub mod read_only;
pub mod readable;
pub mod typed;
pub mod writable;

use raw::MutableRawVec;
pub use read_only::ReadOnlyMutableVec;

/// Stored raw vector whose existing values may be replaced or deleted during an update.
///
/// The mutation state (holes, pending updates) lives here: the wrapped raw vector stays
/// append-only and is reachable through `Deref`.
#[derive(Debug)]
#[must_use]
pub struct MutableVec<V>
where
    V: TypedVec,
{
    vec: V,
    holes: WithPrev<Arc<BTreeSet<usize>>>,
    updated: WithPrev<BTreeMap<usize, V::T>>,
    has_stored_holes: bool,
    published_holes: Arc<RwLock<Arc<BTreeSet<usize>>>>,
}

impl<V> MutableVec<V>
where
    V: TypedVec,
{
    fn from_parts(vec: V, holes: BTreeSet<usize>, has_stored_holes: bool) -> Self {
        let holes = Arc::new(holes);
        Self {
            vec,
            holes: WithPrev::new(Arc::clone(&holes)),
            updated: WithPrev::default(),
            has_stored_holes,
            published_holes: Arc::new(RwLock::new(holes)),
        }
    }

    #[inline(always)]
    pub(crate) fn holes(&self) -> &BTreeSet<usize> {
        &self.holes.current
    }

    #[inline]
    fn mut_holes(&mut self) -> &mut BTreeSet<usize> {
        Arc::make_mut(&mut self.holes.current)
    }

    #[inline]
    fn holes_changed(&self) -> bool {
        let published = self.published_holes.read();
        !Arc::ptr_eq(&self.holes.current, &published) && self.holes() != published.as_ref()
    }

    fn publish_holes(&self) {
        *self.published_holes.write() = Arc::clone(&self.holes.current);
    }

    fn truncate_mutations_at(&mut self, index: usize) {
        if self.holes().last().is_some_and(|&hole| hole >= index) {
            self.mut_holes().split_off(&index);
        }
        if self
            .updated
            .current
            .last_key_value()
            .is_some_and(|(&updated, _)| updated >= index)
        {
            self.updated.current.split_off(&index);
        }
    }

    fn holes_region_name(&self) -> String
    where
        V: AnyVec,
    {
        Self::holes_region_name_with(self.vec.name())
    }

    fn holes_region_name_with(name: &str) -> String {
        format!("{}_holes", vec_region_name_with::<V::I>(name))
    }
}

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    fn install_updates(&mut self, mut updates: BTreeMap<usize, V::T>) {
        let stored_len = self.vec.stored_len();
        let pushed = updates.split_off(&stored_len);
        for (index, value) in pushed {
            self.vec.pushed_mut()[index - stored_len] = value;
        }

        if self.updated.current.is_empty() {
            self.updated.current = updates;
        } else {
            self.updated.current.append(&mut updates);
        }
    }

    /// Replaces existing values: `IndexTooHigh` for an index past the length. Sorted input lets
    /// `BTreeMap` build the mutation tree in linear time.
    pub fn update_many(&mut self, updates: impl IntoIterator<Item = (V::I, V::T)>) -> Result<()> {
        let updates = self.checked_updates(updates)?;
        self.apply_updates(updates);
        Ok(())
    }

    /// [`Self::update_many`] for replacements whose stored values all equal `previous`: their
    /// undo records take it instead of reading storage.
    pub fn update_many_with_previous(
        &mut self,
        updates: impl IntoIterator<Item = (V::I, V::T)>,
        previous: V::T,
    ) -> Result<()>
    where
        V::T: PartialEq,
    {
        let updates = self.checked_updates(updates)?;
        if self.saved_stamped_changes() > 0 {
            let rollback_len = self.vec.rollback_len();
            for &index in updates.keys().take_while(|&&index| index < rollback_len) {
                // Pending updates and holes leave storage unknown: the write reads it.
                if self.updated.current.contains_key(&index) || self.holes().contains(&index) {
                    continue;
                }
                debug_assert!(
                    V::read_stored(&self.vec.reader(), index) == previous,
                    "{}: stored value at {index} differs from the given previous value",
                    self.vec.name()
                );
                self.updated
                    .previous
                    .entry(index)
                    .or_insert_with(|| previous.clone());
            }
        }
        self.apply_updates(updates);
        Ok(())
    }

    fn checked_updates(
        &self,
        updates: impl IntoIterator<Item = (V::I, V::T)>,
    ) -> Result<BTreeMap<usize, V::T>> {
        self.header().check_writable()?;
        let updates: BTreeMap<_, _> = updates
            .into_iter()
            .map(|(index, value)| (index.to_usize(), value))
            .collect();
        if let Some((&last, _)) = updates.last_key_value()
            && last >= self.vec.len()
        {
            return Err(Error::IndexTooHigh {
                index: last,
                len: self.vec.len(),
                name: self.vec.name().to_string(),
            });
        }
        Ok(updates)
    }

    fn apply_updates(&mut self, updates: BTreeMap<usize, V::T>) {
        // Unshare the holes only when a replacement fills one.
        let holes = self.holes();
        if !holes.is_empty() && updates.keys().any(|index| holes.contains(index)) {
            let holes = self.mut_holes();
            for index in updates.keys() {
                holes.remove(index);
            }
        }
        self.install_updates(updates);
    }

    /// Fills the lowest available indexes, then appends the remaining values.
    pub(crate) fn fill_holes_or_push_many(
        &mut self,
        mut values: impl ExactSizeIterator<Item = V::T>,
    ) -> Vec<V::I> {
        self.header().assert_writable();
        let hole_count = self.holes().len().min(values.len());
        let mut indices = Vec::with_capacity(values.len());
        if hole_count > 0 {
            let holes = self.mut_holes();
            indices.extend(
                (0..hole_count).map(|_| holes.pop_first().expect("bounded MutableVec hole batch")),
            );
        }

        let updates = indices
            .iter()
            .copied()
            .zip(values.by_ref().take(hole_count))
            .collect();
        self.install_updates(updates);

        let append_start = self.vec.len();
        let append_count = values.len();
        self.vec.reserve_pushed(append_count);
        for value in values {
            self.vec.push(value);
        }
        indices.extend(append_start..append_start + append_count);
        indices.into_iter().map(V::I::from).collect()
    }

    fn write_inner(&mut self, preserve_rollback_values: bool) -> Result<bool> {
        let guard = self.header().begin_write()?;
        let mut changed = self.vec.write()?;

        if !self.updated.current.is_empty() {
            let updated = mem::take(&mut self.updated.current);
            if preserve_rollback_values && self.vec.saved_stamped_changes() > 0 {
                let rollback_len = self.vec.rollback_len();
                let reader = self.vec.reader();
                let previous = &mut self.updated.previous;
                for &index in updated.keys().take_while(|&&index| index < rollback_len) {
                    previous
                        .entry(index)
                        .or_insert_with(|| V::read_stored(&reader, index));
                }
            }
            self.vec.write_updates(updated)?;
            changed = true;
        }

        if self.holes_changed() {
            if !self.holes().is_empty() {
                let region = self
                    .vec
                    .db()
                    .create_region_if_needed(&self.holes_region_name())?;
                let mut bytes = Vec::with_capacity(self.holes().len() * size_of::<usize>());
                for index in self.holes() {
                    bytes.extend(index.to_bytes());
                }
                region.truncate_write(0, &bytes)?;
                self.has_stored_holes = true;
                changed = true;
            } else if self.has_stored_holes {
                let db = self.vec.db();
                let name = self.holes_region_name();
                debug!("{}: removing holes region '{}'", db, name);
                db.remove_region(&name)?;
                self.has_stored_holes = false;
                changed = true;
            }
            self.publish_holes();
        }

        guard.finish(Ok(changed))
    }

    pub(crate) fn update_at(&mut self, index: usize, value: V::T) -> Result<()> {
        self.header().check_writable()?;
        let stored_len = self.vec.stored_len();
        if index >= stored_len {
            let Some(slot) = self.vec.pushed_mut().get_mut(index - stored_len) else {
                return Err(Error::IndexTooHigh {
                    index,
                    len: stored_len,
                    name: self.vec.name().to_string(),
                });
            };
            *slot = value;
            return Ok(());
        }

        if self.holes().contains(&index) {
            self.mut_holes().remove(&index);
        }
        self.updated.current.insert(index, value);
        Ok(())
    }
}

impl<V> Deref for MutableVec<V>
where
    V: TypedVec,
{
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}
