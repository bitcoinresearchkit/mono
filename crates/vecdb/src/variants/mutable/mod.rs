use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
    sync::Arc,
};

use parking_lot::RwLock;

use log::debug;

use crate::{AnyVec, Bytes, Error, Result, TypedVec, VecIndex, WithPrev, vec_region_name_with};

mod any_stored_vec;
mod any_vec;
mod importable;
mod raw;
mod read_only;
mod readable;
mod typed;
mod writable;

use raw::MutableRawVec;
pub use read_only::ReadOnlyMutableVec;

/// Stored raw vector whose existing values may be replaced during an update.
///
/// Mutation state lives here, leaving the wrapped raw vector lean and
/// append-only. The wrapped vector remains directly available for reads.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
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
    #[inline]
    fn new(vec: V) -> Self {
        Self::from_parts(vec, BTreeSet::new(), false)
    }

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
    fn current_holes(&self) -> &BTreeSet<usize> {
        self.holes.current()
    }

    #[inline]
    fn mut_holes(&mut self) -> &mut BTreeSet<usize> {
        Arc::make_mut(self.holes.current_mut())
    }

    #[inline(always)]
    fn current_updated(&self) -> &BTreeMap<usize, V::T> {
        self.updated.current()
    }

    #[inline]
    fn mut_updated(&mut self) -> &mut BTreeMap<usize, V::T> {
        self.updated.current_mut()
    }

    #[inline(always)]
    fn previous_updated(&self) -> &BTreeMap<usize, V::T> {
        self.updated.previous()
    }

    #[inline]
    fn holes_changed(&self) -> bool {
        let published = self.published_holes.read();
        !Arc::ptr_eq(self.holes.current(), &published) && self.current_holes() != published.as_ref()
    }

    fn publish_holes(&self) {
        *self.published_holes.write() = Arc::clone(self.holes.current());
    }

    fn truncate_mutations_at(&mut self, index: usize) {
        if self
            .current_holes()
            .last()
            .is_some_and(|&hole| hole >= index)
        {
            Arc::make_mut(self.holes.current_mut()).split_off(&index);
        }
        if self
            .current_updated()
            .last_key_value()
            .is_some_and(|(&updated, _)| updated >= index)
        {
            self.mut_updated().split_off(&index);
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

        if self.current_updated().is_empty() {
            *self.updated.current_mut() = updates;
        } else {
            self.mut_updated().append(&mut updates);
        }
    }

    /// Applies a batch of replacements. Sorted input lets `BTreeMap` build the
    /// mutation tree in linear time.
    pub fn update_many(&mut self, updates: impl IntoIterator<Item = (V::I, V::T)>) -> Result<()> {
        let updates: BTreeMap<_, _> = updates
            .into_iter()
            .map(|(index, value)| (index.to_usize(), value))
            .collect();
        let Some((&last, _)) = updates.last_key_value() else {
            return Ok(());
        };
        let len = self.vec.len();
        if last >= len {
            return Err(Error::IndexTooHigh {
                index: last,
                len,
                name: self.vec.name().to_string(),
            });
        }

        if !self.current_holes().is_empty() {
            let holes = self.mut_holes();
            for index in updates.keys() {
                holes.remove(index);
            }
        }
        self.install_updates(updates);
        Ok(())
    }

    /// Fills the lowest available indexes, then appends the remaining values.
    pub fn fill_holes_or_push_many(
        &mut self,
        mut values: impl ExactSizeIterator<Item = V::T>,
    ) -> Vec<V::I> {
        let hole_count = self.current_holes().len().min(values.len());
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
        let mut changed = self.vec.write()?;

        if !self.current_updated().is_empty() {
            let updated = self.updated.take_current();
            if preserve_rollback_values && self.vec.saved_stamped_changes() > 0 {
                let rollback_len = self.vec.rollback_len();
                let reader = self.vec.reader();
                let previous = self.updated.previous_mut();
                for &index in updated.keys().take_while(|&&index| index < rollback_len) {
                    previous
                        .entry(index)
                        .or_insert_with(|| V::read_stored(&reader, index));
                }
            }
            self.vec.write_updates(updated);
            changed = true;
        }

        if self.holes_changed() {
            if !self.current_holes().is_empty() {
                self.has_stored_holes = true;
                let region = self
                    .vec
                    .db()
                    .create_region_if_needed(&self.holes_region_name())?;
                let mut bytes = Vec::with_capacity(self.current_holes().len() * size_of::<usize>());
                for index in self.current_holes() {
                    bytes.extend(index.to_bytes());
                }
                region.truncate_write(0, &bytes)?;
                changed = true;
            } else if self.has_stored_holes {
                self.has_stored_holes = false;
                let db = self.vec.db();
                let name = self.holes_region_name();
                debug!("{}: removing holes region '{}'", db, name);
                db.remove_region(&name)?;
                changed = true;
            }
            self.publish_holes();
        }

        Ok(changed)
    }

    fn update_value_at(&mut self, index: usize, value: V::T) -> Result<()> {
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

        if self.current_holes().contains(&index) {
            self.mut_holes().remove(&index);
        }
        self.mut_updated().insert(index, value);
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
