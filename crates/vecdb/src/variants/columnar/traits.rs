use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use rawdb::{Database, Region};

use crate::{
    AnyStoredVec, AnyVec, Error, Header, ImportOptions, ImportableVec, ReadableBoxedVec,
    ReadableCloneableVec, Stamp, StoredVec, TypedVec, Version, WritableVec, short_type_name,
};

use super::{ColumnId, ColumnarVec, LazyColumnVec, ReadOnlyColumnarVec, ReadableColumnarVec};

impl<V, C> ImportableVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Self::import_with((db, name, version).into())
    }

    fn import_with(options: ImportOptions) -> crate::Result<Self> {
        Self::import_inner(options, false)
    }

    fn forced_import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Self::forced_import_with((db, name, version).into())
    }

    fn forced_import_with(options: ImportOptions) -> crate::Result<Self> {
        Self::import_inner(options, true)
    }
}

impl<V, C> AnyVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn version(&self) -> Version {
        self.vec.version()
    }

    fn name(&self) -> &str {
        self.vec.name()
    }

    fn len(&self) -> usize {
        self.stored_rows + self.pushed.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.vec.index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<C::Row<V::T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<C::Row<V::T>>()
    }

    fn region_names(&self) -> Vec<String> {
        self.vec.region_names()
    }
}

impl<V, C> AnyVec for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn version(&self) -> Version {
        self.vec.version()
    }

    fn name(&self) -> &str {
        self.vec.name()
    }

    fn len(&self) -> usize {
        self.visible_rows.get()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.vec.index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<C::Row<V::T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<C::Row<V::T>>()
    }

    fn region_names(&self) -> Vec<String> {
        self.vec.region_names()
    }
}

impl<S, C> AnyVec for LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    fn version(&self) -> Version {
        self.base_version
            .combine(self.source.version())
            .combine(Version::from(self.column.index() + 1))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.source.index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<S::T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<S::T>()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<V, C> TypedVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    type I = V::I;
    type T = C::Row<V::T>;
}

impl<V, C> TypedVec for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    type I = V::I;
    type T = C::Row<V::T>;
}

impl<S, C> TypedVec for LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    type I = S::I;
    type T = S::T;
}

impl<V, C> AnyStoredVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn db_path(&self) -> PathBuf {
        self.vec.db_path()
    }

    fn region(&self) -> &Region {
        self.vec.region()
    }

    fn header(&self) -> &Header {
        self.vec.header()
    }

    fn mut_header(&mut self) -> &mut Header {
        self.vec.mut_header()
    }

    fn saved_stamped_changes(&self) -> u16 {
        self.vec.saved_stamped_changes()
    }

    fn write(&mut self) -> crate::Result<bool> {
        self.write_pending()
    }

    fn db(&self) -> Database {
        self.vec.db()
    }

    fn real_stored_len(&self) -> usize {
        debug_assert!(
            self.vec
                .real_stored_len()
                .is_multiple_of(Self::COLUMN_COUNT)
        );
        self.vec.real_stored_len() / Self::COLUMN_COUNT
    }

    fn stored_len(&self) -> usize {
        self.stored_rows
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        <Self as WritableVec<V::I, C::Row<V::T>>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<V::I, C::Row<V::T>>>::save_rollback_state(self)
    }

    fn serialize_changes(&self) -> crate::Result<Vec<u8>> {
        if !self.pushed.is_empty() || self.stored_rows != self.flat_rows() {
            return Err(Error::InvalidArgument(
                "ColumnarVec changes must be staged before serialization",
            ));
        }
        self.vec.serialize_changes()
    }

    fn remove(self) -> crate::Result<()> {
        self.vec.remove()
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        <Self as WritableVec<V::I, C::Row<V::T>>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> crate::Result<()> {
        <Self as WritableVec<V::I, C::Row<V::T>>>::reset(self)
    }
}

impl<V, C> WritableVec<V::I, C::Row<V::T>> for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn push(&mut self, value: C::Row<V::T>) {
        self.pushed.push(value);
    }

    fn pushed(&self) -> &[C::Row<V::T>] {
        &self.pushed
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        let len = self.len();
        if index >= len {
            return Ok(());
        }
        if index < self.stored_rows {
            self.stored_rows = index;
            self.pushed.clear();
        } else {
            self.pushed.truncate(index - self.stored_rows);
        }
        Ok(())
    }

    fn reset(&mut self) -> crate::Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.stored_rows = 0;
        self.pushed.clear();
        self.vec.reset()?;
        self.visible_rows.set(0);
        Ok(())
    }

    fn reset_unsaved(&mut self) {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.vec.reset_unsaved();
        self.stored_rows = self.flat_rows();
        self.pushed.clear();
        self.visible_rows.set(self.stored_rows);
    }

    fn is_dirty(&self) -> bool {
        self.stored_rows != self.flat_rows() || !self.pushed.is_empty() || self.vec.is_dirty()
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.stage_pending()?;
        self.vec.stamped_write_with_changes(stamp)?;
        self.stored_rows = self.flat_rows();
        self.visible_rows.set(self.stored_rows);
        Ok(())
    }

    fn rollback(&mut self) -> crate::Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.pushed.clear();
        self.vec.rollback()?;
        self.stored_rows = self.flat_rows();
        debug_assert!(self.vec.stored_len().is_multiple_of(Self::COLUMN_COUNT));
        self.visible_rows
            .set(self.vec.stored_len() / Self::COLUMN_COUNT);
        Ok(())
    }

    fn find_rollback_files(&self) -> crate::Result<BTreeMap<Stamp, PathBuf>> {
        self.vec.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.vec.save_rollback_state();
    }
}

impl<V, C> StoredVec for ColumnarVec<V, C>
where
    V: StoredVec + 'static,
    C: ColumnId,
{
    type ReadOnly = ReadOnlyColumnarVec<V, C>;

    fn read_only_clone(&self) -> Self::ReadOnly {
        ColumnarVec::read_only_clone(self)
    }
}

impl<V, C> ReadableCloneableVec<V::I, C::Row<V::T>> for ColumnarVec<V, C>
where
    V: StoredVec + 'static,
    C: ColumnId,
{
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<V::I, C::Row<V::T>> {
        ReadableBoxedVec::new(self.read_only_clone())
    }
}
