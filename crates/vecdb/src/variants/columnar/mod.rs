use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    Error, ImportOptions, MAX_UNCOMPRESSED_PAGE_SIZE, SharedLen, StoredVec, VecIndex, Version,
};

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

use read::read_rows;
use schema::validate_schema;

const VERSION: Version = Version::new(8);

#[inline]
pub(super) const fn rows_per_block<T>() -> usize {
    MAX_UNCOMPRESSED_PAGE_SIZE / size_of::<T>()
}

/// One logical vector of rows stored as page-sized, column-major blocks in `V`.
///
/// `V` remains an ordinary flat scalar vector. Every complete block of
/// `ROWS_PER_BLOCK` rows is flattened as consecutive scalar column pages:
///
/// ```text
/// [column 0 page][column 1 page] ... [last column page]
/// ```
///
/// The final incomplete block is rebuilt in column-major order whenever it is
/// persisted. Raw and compressed vectors therefore keep their existing write,
/// page, rollback, and compression logic unchanged.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    vec: V,
    stored_rows: usize,
    pushed: Vec<C::Row<V::T>>,
    visible_rows: SharedLen,
    gate: Arc<RwLock<()>>,
}

impl<V, C> ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    pub(super) const COLUMN_COUNT: usize = C::ALL.len();
    pub(super) const ROWS_PER_BLOCK: usize = rows_per_block::<V::T>();

    fn validate_layout() -> crate::Result<Version> {
        let schema = validate_schema::<C>()?;
        if size_of::<V::T>() == 0 || Self::ROWS_PER_BLOCK == 0 {
            return Err(Error::InvalidArgument(
                "ColumnarVec requires at least one non-zero-sized scalar per page",
            ));
        }
        Ok(schema)
    }

    fn validate_flat_len(vec: &V) -> crate::Result<usize> {
        let len = vec.len();
        if !len.is_multiple_of(Self::COLUMN_COUNT) {
            return Err(Error::CorruptedRegion {
                name: vec.name().to_string(),
                region_len: vec.region().meta().len(),
            });
        }
        Ok(len / Self::COLUMN_COUNT)
    }

    fn import_inner(mut options: ImportOptions, forced: bool) -> crate::Result<Self> {
        let schema = Self::validate_layout()?;
        let columns = u32::try_from(Self::COLUMN_COUNT).map_err(|_| Error::Overflow)?;
        options.version = options
            .version
            .combine(VERSION)
            .combine(C::VERSION)
            .combine(schema)
            .combine(Version::new(columns));
        options.initial_capacity = Some(
            options
                .initial_capacity
                .unwrap_or(V::I::INITIAL_CAPACITY)
                .checked_mul(Self::COLUMN_COUNT)
                .ok_or(Error::Overflow)?,
        );
        let mut vec = if forced {
            V::forced_import_with(options)?
        } else {
            V::import_with(options)?
        };

        let stored_rows = match Self::validate_flat_len(&vec) {
            Ok(len) => len,
            Err(_) if forced => {
                vec.reset()?;
                vec.write()?;
                Self::validate_flat_len(&vec)?
            }
            Err(error) => return Err(error),
        };
        Ok(Self {
            vec,
            stored_rows,
            pushed: Vec::new(),
            visible_rows: SharedLen::new(stored_rows),
            gate: Arc::new(RwLock::new(())),
        })
    }

    pub fn read_only_clone(&self) -> ReadOnlyColumnarVec<V, C> {
        ReadOnlyColumnarVec {
            vec: self.vec.read_only_clone(),
            visible_rows: self.visible_rows.clone(),
            gate: Arc::clone(&self.gate),
            columns: std::marker::PhantomData,
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

    #[inline]
    fn flat_rows(&self) -> usize {
        debug_assert!(self.vec.len().is_multiple_of(Self::COLUMN_COUNT));
        self.vec.len() / Self::COLUMN_COUNT
    }

    #[inline]
    fn push_block(vec: &mut V, rows: &[C::Row<V::T>]) {
        for &column in C::ALL {
            for row in rows {
                vec.push(column.get(row).clone());
            }
        }
    }

    /// Moves the logical matrix changes into the wrapped flat vector.
    ///
    /// Only the incomplete height block is read and rebuilt. Completed blocks
    /// already consist of aligned scalar column pages and remain untouched.
    fn stage_pending(&mut self) -> crate::Result<()> {
        let flat_rows = self.flat_rows();
        if self.stored_rows == flat_rows && self.pushed.is_empty() {
            return Ok(());
        }

        let target_rows = self
            .stored_rows
            .checked_add(self.pushed.len())
            .ok_or(Error::Overflow)?;
        let block_start = self.stored_rows / Self::ROWS_PER_BLOCK * Self::ROWS_PER_BLOCK;
        let retained_capacity = if self.stored_rows == block_start {
            0
        } else {
            (target_rows - block_start).min(Self::ROWS_PER_BLOCK)
        };
        let mut retained = Vec::with_capacity(retained_capacity);
        read_rows::<V::I, V::T, V, C>(
            &self.vec,
            flat_rows,
            block_start,
            self.stored_rows,
            &mut retained,
        );

        let mut pushed_from = 0;
        if !retained.is_empty() {
            let take = (Self::ROWS_PER_BLOCK - retained.len()).min(self.pushed.len());
            retained.extend_from_slice(&self.pushed[..take]);
            pushed_from = take;
        }

        let flat_start = block_start
            .checked_mul(Self::COLUMN_COUNT)
            .ok_or(Error::Overflow)?;
        self.vec.truncate_if_needed_at(flat_start)?;
        Self::push_block(&mut self.vec, &retained);
        for block in self.pushed[pushed_from..].chunks(Self::ROWS_PER_BLOCK) {
            Self::push_block(&mut self.vec, block);
        }

        self.stored_rows = target_rows;
        self.pushed.clear();
        debug_assert_eq!(self.vec.len(), target_rows * Self::COLUMN_COUNT);
        Ok(())
    }

    fn write_pending(&mut self) -> crate::Result<bool> {
        let gate = Arc::clone(&self.gate);
        let _guard = gate.write();
        self.stage_pending()?;
        let written = self.vec.write()?;
        self.stored_rows = self.flat_rows();
        self.visible_rows.set(self.stored_rows);
        Ok(written)
    }
}
