use std::{fs, marker::PhantomData, ops::Deref, path::PathBuf, result::Result, sync::Arc};

use rawdb::Database;

use super::{Format, HEADER_OFFSET, Header, ImportOptions, ReadOnlyBaseVec, SharedLen, WithPrev};
use crate::{Error, Result as CrateResult, Stamp, VecIndex, VecValue};

/// Base storage vector with fields common to all stored vector implementations.
///
/// Derefs to [`ReadOnlyBaseVec`] for read-only access to region, header, name,
/// stored_len, and version. Write state (pushed, rollback) lives here.
#[derive(Debug)]
pub struct ReadWriteBaseVec<I, T> {
    pub read_only: ReadOnlyBaseVec<I, T>,
    pub pushed: WithPrev<Vec<T>>,
    pub previous_stored_len: usize,
    pub saved_stamped_changes: u16,
}

impl<I, T> Deref for ReadWriteBaseVec<I, T> {
    type Target = ReadOnlyBaseVec<I, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.read_only
    }
}

impl<I, T> ReadWriteBaseVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    pub fn import(options: ImportOptions, format: Format) -> CrateResult<Self> {
        let initial_capacity = options.initial_capacity.unwrap_or(I::INITIAL_CAPACITY);
        let region = options
            .db
            .create_region_if_needed(&vec_region_name_with::<I>(options.name))?;

        let region_len = region.meta().len();
        if region_len > 0 && region_len < HEADER_OFFSET {
            return Err(Error::CorruptedRegion {
                name: region.meta().id().to_string(),
                region_len,
            });
        }

        if region_len == 0 && initial_capacity > 0 {
            let capacity = initial_capacity
                .checked_mul(size_of::<T>())
                .and_then(|bytes| bytes.checked_add(HEADER_OFFSET))
                .ok_or(Error::Overflow)?;
            region.reserve_capacity(capacity)?;
        }

        let header = if region_len == 0 {
            Header::create_and_write(&region, options.version, format)?
        } else {
            Header::import_and_verify(&region, options.version, format)?
        };

        Ok(Self {
            read_only: ReadOnlyBaseVec {
                region,
                header,
                name: Arc::from(options.name),
                stored_len: SharedLen::default(),
                phantom: PhantomData,
            },
            pushed: WithPrev::default(),
            previous_stored_len: 0,
            saved_stamped_changes: options.saved_stamped_changes,
        })
    }

    #[inline]
    pub fn read_only_base(&self) -> ReadOnlyBaseVec<I, T> {
        self.read_only.clone()
    }

    #[inline]
    pub fn pushed(&self) -> &[T] {
        self.pushed.current()
    }

    #[inline]
    pub fn mut_pushed(&mut self) -> &mut Vec<T> {
        self.header.assert_writable();
        self.pushed.current_mut()
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.header.assert_writable();
        self.pushed.current_mut().reserve(additional);
    }

    #[inline]
    pub fn prev_pushed(&self) -> &[T] {
        self.pushed.previous()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stored_len() + self.pushed().len()
    }

    #[inline]
    pub fn update_stored_len(&self, val: usize) {
        self.read_only.stored_len.set(val);
    }

    #[inline]
    pub fn prev_stored_len(&self) -> usize {
        self.previous_stored_len
    }

    #[inline(always)]
    pub fn mut_prev_stored_len(&mut self) -> &mut usize {
        &mut self.previous_stored_len
    }

    #[inline(always)]
    pub fn saved_stamped_changes(&self) -> u16 {
        self.saved_stamped_changes
    }

    #[inline]
    pub fn mut_header(&mut self) -> &mut Header {
        &mut self.read_only.header
    }

    #[inline]
    pub fn db(&self) -> Database {
        self.region.db()
    }

    #[inline]
    pub fn db_path(&self) -> PathBuf {
        self.db().path().to_path_buf()
    }

    pub fn write_header_if_needed(&mut self) -> CrateResult<bool> {
        if self.read_only.header.modified() {
            self.read_only.header.write(&self.read_only.region)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn index_to_name(&self) -> String {
        vec_region_name(&self.name, I::to_string())
    }

    pub fn remove(self) -> CrateResult<()> {
        self.read_only.region.remove()?;
        Ok(())
    }

    /// Tight pointer loop for LLVM vectorization.
    #[inline]
    pub fn fold_pushed<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let start = from.max(stored_len);
        if start >= to {
            return init;
        }
        let pushed = self.pushed();
        let slice_from = start - stored_len;
        let slice_to = (to - stored_len).min(pushed.len());
        let mut acc = init;
        let mut i = slice_from;
        while i < slice_to {
            acc = f(acc, pushed[i].clone());
            i += 1;
        }
        acc
    }

    #[inline]
    pub fn try_fold_pushed<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let stored_len = self.stored_len();
        let start = from.max(stored_len);
        if start >= to {
            return Ok(init);
        }
        let pushed = self.pushed();
        let mut acc = init;
        for v in &pushed[(start - stored_len)..(to - stored_len).min(pushed.len())] {
            acc = f(acc, v.clone())?;
        }
        Ok(acc)
    }

    /// Returns true if stored_len needs updating to `index`.
    pub fn truncate_pushed(&mut self, index: usize) -> bool {
        let stored_len = self.stored_len();
        let len = stored_len + self.pushed().len();

        if index >= len {
            return false;
        }

        if index <= stored_len {
            self.pushed.current_mut().clear();
        } else {
            self.pushed.current_mut().truncate(index - stored_len);
        }

        index < stored_len
    }

    pub fn reset_base(&mut self) -> CrateResult<()> {
        self.header.check_writable()?;
        self.pushed.clear();
        self.read_only.stored_len.set(0);
        self.previous_stored_len = 0;
        self.read_only.header.update_stamp(Stamp::default());

        let changes_path = self.changes_path();
        if changes_path.exists() {
            fs::remove_dir_all(&changes_path)?;
        }

        Ok(())
    }

    pub fn reset_unsaved_base(&mut self) {
        self.header.assert_writable();
        self.pushed.current_mut().clear();
    }
}

/// Region name for a vec of type `I`, e.g. `"height/Height"`.
pub fn vec_region_name_with<I: VecIndex>(name: &str) -> String {
    vec_region_name(name, I::to_string())
}

/// Formats a vec's region name as `{name}/{index}`.
pub fn vec_region_name(name: &str, index: &str) -> String {
    format!("{name}/{index}")
}
