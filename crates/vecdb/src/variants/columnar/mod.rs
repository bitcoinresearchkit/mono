use std::{marker::PhantomData, mem, sync::Arc};

use log::warn;
use parking_lot::RwLock;
use rawdb::{Error as RawDbError, Region, RegionGroup};

use crate::{Error, ImportOptions, Result, SharedLen, StoredVec, Version};

mod column;
mod lazy;
mod read;
mod read_only;
mod schema;
mod sum;
mod traits;

pub use column::*;
pub use lazy::*;
pub use read_only::*;
pub use schema::*;
pub use sum::*;

use schema::validate_schema;

const VERSION: Version = Version::new(10);

/// One logical row vector backed by one contiguous stored vector per column.
///
/// All column vectors always have the same logical length. If their persisted
/// lengths or stamps disagree during import, the complete vector is reset.
///
/// Every column remains an ordinary `V`. Rawdb groups all regions owned by the
/// columns into one ordered allocation and relocates the complete allocation
/// whenever any member grows.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    name: Arc<str>,
    columns: Vec<V>,
    read_only_columns: Arc<[V::ReadOnly]>,
    pushed: Vec<C::Row<V::T>>,
    visible_rows: SharedLen,
    gate: Arc<RwLock<()>>,
    group: RegionGroup,
}

impl<V, C> ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    pub(super) const COLUMN_COUNT: usize = C::ALL.len();

    fn column_name(name: &str, column: C) -> String {
        format!("{name}_{column:?}")
    }

    fn import_inner(mut options: ImportOptions, forced: bool) -> Result<Self> {
        let schema = validate_schema::<C>()?;
        let column_count = u32::try_from(Self::COLUMN_COUNT).map_err(|_| Error::Overflow)?;
        options.version = options
            .version
            .combine(VERSION)
            .combine(C::VERSION)
            .combine(schema)
            .combine(Version::new(column_count));

        let mut columns = Vec::with_capacity(Self::COLUMN_COUNT);
        for &column in C::ALL {
            let column_name = Self::column_name(options.name, column);
            let column_options = ImportOptions {
                name: &column_name,
                ..options
            };
            columns.push(if forced {
                V::forced_import_with(column_options)?
            } else {
                V::import_with(column_options)?
            });
        }

        if let Err(error) = Self::validate_columns(&columns) {
            for column in &mut columns {
                column.reset()?;
                column.write()?;
            }
            Self::validate_columns(&columns)?;
            warn!(
                "Reset {} because its columns disagreed: {error}",
                options.name
            );
        }

        let db = options.db;
        let mut regions = Vec::<Region>::new();
        for column in &columns {
            for name in column.region_names() {
                regions.push(db.get_region(&name).ok_or(RawDbError::RegionNotFound)?);
            }
        }
        let group = db.group_regions(&regions)?;
        let stored_rows = columns[0].stored_len();
        let read_only_columns = columns
            .iter()
            .map(StoredVec::read_only_clone)
            .collect::<Arc<[_]>>();

        Ok(Self {
            name: Arc::from(options.name),
            columns,
            read_only_columns,
            pushed: Vec::new(),
            visible_rows: SharedLen::new(stored_rows),
            gate: Arc::new(RwLock::new(())),
            group,
        })
    }

    fn validate_columns(columns: &[V]) -> Result<()> {
        let first = &columns[0];
        let expected_len = first.len();
        let expected_real_len = first.real_stored_len();
        let expected_stamp = first.stamp();
        for column in &columns[1..] {
            if column.len() != expected_len {
                return Err(Error::WrongLength {
                    received: column.len(),
                    expected: expected_len,
                });
            }
            if column.real_stored_len() != expected_real_len {
                return Err(Error::WrongLength {
                    received: column.real_stored_len(),
                    expected: expected_real_len,
                });
            }
            if column.stamp() != expected_stamp {
                return Err(Error::StampMismatch {
                    file: column.stamp(),
                    vec: expected_stamp,
                });
            }
        }
        Ok(())
    }

    #[inline]
    pub(super) fn first(&self) -> &V {
        &self.columns[0]
    }

    #[inline]
    pub(super) fn first_mut(&mut self) -> &mut V {
        &mut self.columns[0]
    }

    #[inline]
    pub(super) fn stored_rows(&self) -> usize {
        let len = self.first().stored_len();
        debug_assert!(self.columns.iter().all(|column| column.stored_len() == len));
        len
    }

    fn stage_pending(&mut self) {
        let pushed = mem::take(&mut self.pushed);
        if pushed.is_empty() {
            return;
        }
        for row in &pushed {
            for &column in C::ALL {
                self.columns[column.index()].push(column.get(row).clone());
            }
        }
    }

    fn publish_stored_rows(&self) {
        let len = self.stored_rows();
        debug_assert!(self.columns.iter().all(|column| column.len() == len));
        self.visible_rows.set(len);
    }

    fn write_pending(&mut self) -> Result<bool> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.stage_pending();
        let mut written = false;
        for column in &mut self.columns {
            written |= column.write()?;
        }
        self.publish_stored_rows();
        Ok(written)
    }

    pub fn read_only_clone(&self) -> ReadOnlyColumnarVec<V, C> {
        ReadOnlyColumnarVec {
            name: Arc::clone(&self.name),
            columns: Arc::clone(&self.read_only_columns),
            visible_rows: self.visible_rows.clone(),
            gate: Arc::clone(&self.gate),
            column_ids: PhantomData,
        }
    }

    /// Returns an owned read-only view of one persisted scalar column.
    /// Uncommitted rows become visible to the view after `write()`.
    pub fn column(
        &self,
        name: &str,
        version: Version,
        column: C,
    ) -> LazyColumnVec<ReadOnlyColumnarVec<V, C>, C> {
        self.read_only_clone().column(name, version, column)
    }

    /// Returns a lazy sum of selected persisted columns.
    /// Uncommitted rows become visible to the view after `write()`.
    pub fn sum_columns(
        &self,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> LazyColumnSumVec<ReadOnlyColumnarVec<V, C>, C> {
        LazyColumnSumVec::new(name, version, self.read_only_clone(), columns)
    }

    pub fn reserve_pushed(&mut self, additional: usize) {
        self.pushed.reserve(additional);
    }
}
