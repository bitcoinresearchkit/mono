use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use rawdb::{Database, Region};

use crate::{
    AnyStoredVec, AnyVec, Error, Header, ImportOptions, ImportableVec, ReadableBoxedVec,
    ReadableCloneableVec, Result, Stamp, StoredVec, TypedVec, Version, WritableVec,
    short_type_name,
};

use super::{ColumnId, ColumnarVec, LazyColumnVec, ReadOnlyColumnarVec, ReadableColumnarVec};

impl<V, C> ImportableVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
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

impl<V, C> AnyVec for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn version(&self) -> Version {
        self.first().version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.stored_rows() + self.pushed.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.first().index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<C::Row<V::T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<C::Row<V::T>>()
    }

    fn region_names(&self) -> Vec<String> {
        self.columns.iter().flat_map(AnyVec::region_names).collect()
    }
}

impl<V, C> AnyVec for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn version(&self) -> Version {
        self.columns[0].version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.visible_rows.get()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.columns[0].index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<C::Row<V::T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<C::Row<V::T>>()
    }

    fn region_names(&self) -> Vec<String> {
        self.columns.iter().flat_map(AnyVec::region_names).collect()
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
        self.first().db_path()
    }

    fn region(&self) -> &Region {
        self.first().region()
    }

    fn header(&self) -> &Header {
        self.first().header()
    }

    fn mut_header(&mut self) -> &mut Header {
        self.first_mut().mut_header()
    }

    fn saved_stamped_changes(&self) -> u16 {
        self.first().saved_stamped_changes()
    }

    fn write(&mut self) -> Result<bool> {
        self.write_pending()
    }

    fn flush(&mut self) -> Result<()> {
        let db = self.db();
        if self.write()? {
            db.flush()?;
        }
        Ok(())
    }

    fn db(&self) -> Database {
        self.first().db()
    }

    fn real_stored_len(&self) -> usize {
        let len = self.first().real_stored_len();
        debug_assert!(
            self.columns
                .iter()
                .all(|column| column.real_stored_len() == len)
        );
        len
    }

    fn stored_len(&self) -> usize {
        self.stored_rows()
    }

    fn update_stamp(&mut self, stamp: Stamp) {
        for column in &mut self.columns {
            column.update_stamp(stamp);
        }
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        <Self as WritableVec<V::I, C::Row<V::T>>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<V::I, C::Row<V::T>>>::save_rollback_state(self)
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        if !self.pushed.is_empty() {
            return Err(Error::InvalidArgument(
                "ColumnarVec changes must be staged before serialization",
            ));
        }
        let mut combined = Vec::new();
        for column in &self.columns {
            let changes = column.serialize_changes()?;
            combined.extend_from_slice(
                &u64::try_from(changes.len())
                    .map_err(|_| Error::Overflow)?
                    .to_le_bytes(),
            );
            combined.extend(changes);
        }
        Ok(combined)
    }

    fn remove(self) -> Result<()> {
        let Self {
            columns,
            read_only_columns,
            group,
            ..
        } = self;
        drop(read_only_columns);
        drop(group);
        for column in columns {
            column.remove()?;
        }
        Ok(())
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        <Self as WritableVec<V::I, C::Row<V::T>>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> Result<()> {
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

    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        let len = self.len();
        if index >= len {
            return Ok(());
        }

        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        let stored_rows = self.stored_rows();
        self.visible_rows.set(index.min(stored_rows));
        if index < stored_rows {
            self.pushed.clear();
            for column in &mut self.columns {
                column.truncate_if_needed_at(index)?;
            }
        } else {
            self.pushed.truncate(index - stored_rows);
        }
        self.publish_stored_rows();
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.visible_rows.set(0);
        self.pushed.clear();
        for column in &mut self.columns {
            column.reset()?;
        }
        Ok(())
    }

    fn reset_unsaved(&mut self) {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.pushed.clear();
        for column in &mut self.columns {
            column.reset_unsaved();
        }
        self.publish_stored_rows();
    }

    fn is_dirty(&self) -> bool {
        !self.pushed.is_empty() || self.columns.iter().any(WritableVec::is_dirty)
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.stage_pending();
        for column in &mut self.columns {
            column.stamped_write_with_changes(stamp)?;
        }
        self.publish_stored_rows();
        Ok(())
    }

    fn rollback(&mut self) -> Result<()> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.visible_rows.set(0);
        self.pushed.clear();
        for column in &mut self.columns {
            column.rollback()?;
        }
        Self::validate_columns(&self.columns)?;
        self.publish_stored_rows();
        Ok(())
    }

    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>> {
        let files = self.first().find_rollback_files()?;
        for column in &self.columns[1..] {
            let other = column.find_rollback_files()?;
            debug_assert!(files.keys().eq(other.keys()));
        }
        Ok(files)
    }

    fn save_rollback_state(&mut self) {
        for column in &mut self.columns {
            column.save_rollback_state();
        }
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
