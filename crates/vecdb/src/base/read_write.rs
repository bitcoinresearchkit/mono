use std::{fmt, fs, marker::PhantomData, ops::Deref, path::PathBuf, sync::Arc};

use log::Level;
use parking_lot::Mutex;
use rawdb::Database;

use super::{Format, HEADER_OFFSET, Header, ImportOptions, ReadOnlyBaseVec, SharedLen, WithPrev};
use crate::{Error, Result, Stamp, VecIndex, VecValue};

/// Largest pushed allocation kept across writes.
const RETAINED_PUSHED_BYTES: usize = 128;

/// Writer state (pushed values, rollback) over a [`ReadOnlyBaseVec`], which it derefs to.
#[derive(Debug)]
pub struct ReadWriteBaseVec<I, T> {
    pub read_only: ReadOnlyBaseVec<I, T>,
    pub pushed: WithPrev<Vec<T>>,
    pub previous_stored_len: usize,
    pub saved_stamped_changes: u16,
    /// See [`crate::ImportOptions::with_unread_updates`].
    pub unread_updates: bool,
    /// A rollback's or a reset's state is not written yet: unsaved changes cancel back to it
    /// (`previous_stored_len` and the previous pushed values), not to the disk.
    pub unwritten_baseline: bool,
    /// Sorted stamps of the change files, once listed: `None` until then or after an error.
    pub(super) change_stamps: Mutex<Option<Vec<Stamp>>>,
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
    pub fn import(options: ImportOptions, format: Format) -> Result<Self> {
        let region = options
            .db
            .create_region_if_needed(&vec_region_name_with::<I>(options.name))?;

        let region_len = region.meta().byte_len();
        if region_len > 0 && region_len < HEADER_OFFSET {
            return Err(Error::CorruptedRegion {
                name: region.meta().id().to_string(),
                region_len,
            });
        }

        if region_len == 0 && I::INITIAL_CAPACITY > 0 {
            let capacity = I::INITIAL_CAPACITY
                .checked_mul(size_of::<T>())
                .and_then(|bytes| bytes.checked_add(HEADER_OFFSET))
                .ok_or(Error::Overflow)?;
            region.reserve_capacity(capacity)?;
        }

        const { assert!(size_of::<T>() <= u16::MAX as usize) };
        let value_size = size_of::<T>() as u16;
        let header = if region_len == 0 {
            Header::create_and_write(&region, options.version, format, value_size)?
        } else {
            Header::import_and_verify(&region, options.version, format, value_size)?
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
            unread_updates: options.unread_updates,
            unwritten_baseline: false,
            change_stamps: Mutex::default(),
        })
    }

    #[inline]
    pub fn pushed(&self) -> &[T] {
        &self.pushed.current
    }

    #[inline]
    pub fn mut_pushed(&mut self) -> &mut Vec<T> {
        // Every push lands here: a failed write already stopped the caller with an error.
        debug_assert!(
            self.header.check_writable().is_ok(),
            "vector cannot continue after a failed write"
        );
        &mut self.pushed.current
    }

    /// Takes back the written-out pushed buffer, emptied by [`clear_pushed`].
    #[inline]
    pub fn recycle_pushed(&mut self, mut written: Vec<T>) {
        debug_assert!(self.pushed().is_empty());
        clear_pushed(&mut written);
        self.pushed.current = written;
    }

    #[inline]
    pub fn prev_pushed(&self) -> &[T] {
        &self.pushed.previous
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stored_len() + self.pushed().len()
    }

    #[inline]
    pub fn db(&self) -> Database {
        self.region.db()
    }

    #[inline]
    pub fn db_path(&self) -> PathBuf {
        self.db().path().to_path_buf()
    }

    pub fn write_header_if_needed(&mut self) -> Result<bool> {
        if self.read_only.header.modified() {
            self.read_only.header.write(&self.read_only.region)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn region_name(&self) -> String {
        vec_region_name(&self.name, I::to_string())
    }

    pub fn remove(self) -> Result<()> {
        self.read_only.region.remove()?;
        Ok(())
    }

    /// Returns true if stored_len needs updating to `index`.
    pub fn truncate_pushed(&mut self, index: usize) -> bool {
        let stored_len = self.stored_len();
        let len = stored_len + self.pushed().len();

        if index >= len {
            return false;
        }

        if index <= stored_len {
            self.pushed.current.clear();
        } else {
            self.pushed.current.truncate(index - stored_len);
        }

        index < stored_len
    }

    /// Empties the vec until written: unsaved changes cancel back to empty, never to the rows
    /// still on disk (a version reset must not resurrect them).
    pub fn reset_base(&mut self) -> Result<()> {
        self.header.check_writable()?;
        self.pushed.clear();
        self.read_only.stored_len.set(0);
        self.previous_stored_len = 0;
        self.unwritten_baseline = true;
        self.read_only.header.update_stamp(Stamp::default());
        self.forget_change_files();
        remove_changes_with::<I>(&self.db(), &self.name)
    }

    /// Drops unsaved changes: back to an unwritten rollback's or reset's state, else to the
    /// `written_len` rows on disk.
    pub fn reset_unsaved_base(&mut self, written_len: usize) {
        self.header.assert_writable();
        if self.unwritten_baseline {
            self.pushed.restore();
            self.read_only.stored_len.set(self.previous_stored_len);
        } else {
            self.pushed.current.clear();
            self.read_only.stored_len.set(written_len);
        }
    }
}

/// Empties a written-out pushed buffer: a small per-update allocation is kept for the next
/// update, a bulk one is freed.
#[inline]
pub(crate) fn clear_pushed<T>(pushed: &mut Vec<T>) {
    if pushed.capacity() * size_of::<T>() <= RETAINED_PUSHED_BYTES {
        pushed.clear();
    } else {
        *pushed = Vec::new();
    }
}

/// Resets a vec that import rejected with `error` (an import-time reset): removes its region, the
/// `extra` regions it cannot be read without and its rollback history, so a later rollback never
/// walks into change files of the vec's previous life. Logs what was discarded.
pub(crate) fn reset_vec_with<I: VecIndex>(
    db: &Database,
    name: &str,
    error: &Error,
    extra: &[&str],
) -> Result<()> {
    let vec_region = vec_region_name_with::<I>(name);
    let bytes = |region: &str| {
        db.region(region)
            .map_or(0, |region| region.meta().byte_len())
    };
    let stamp = db
        .region(&vec_region)
        .and_then(|region| Header::read_unverified(&region).ok())
        .map(|header| header.stamp());
    let extras: String = extra
        .iter()
        .map(|region| format!(", region {region} ({} bytes)", bytes(region)))
        .collect();
    let changes = fs::read_dir(changes_path_with::<I>(db, name)).map_or(0, Iterator::count);
    log_reset(
        db,
        error,
        format_args!(
            "{name}: discarding region {vec_region} ({} bytes, header stamp {stamp:?}){extras} and {changes} change files, now empty (len 0, stamp 0)",
            bytes(&vec_region)
        ),
    );
    db.remove_region_if_exists(&vec_region)?;
    for region in extra {
        db.remove_region_if_exists(region)?;
    }
    remove_changes_with::<I>(db, name)
}

/// Logs stored data an import is about to discard, after `error` rejected it. A new version,
/// format or value size is a deliberate reset that an upgrade repeats for every vec (debug);
/// any other cause is a torn or corrupt vec, whose data is lost (error).
pub(crate) fn log_reset(db: &Database, error: &Error, discarded: fmt::Arguments) {
    let level = match error {
        Error::DifferentVersion { .. }
        | Error::DifferentFormat { .. }
        | Error::DifferentValueSize { .. } => Level::Debug,
        _ => Level::Error,
    };
    log::log!(
        level,
        "{}: import reset after {error}: {discarded}",
        db.path().display()
    );
}

fn remove_changes_with<I: VecIndex>(db: &Database, name: &str) -> Result<()> {
    let changes_path = changes_path_with::<I>(db, name);
    if changes_path.exists() {
        fs::remove_dir_all(changes_path)?;
    }
    Ok(())
}

/// Rollback history directory of a vec of type `I`.
pub(crate) fn changes_path_with<I: VecIndex>(db: &Database, name: &str) -> PathBuf {
    db.path()
        .join("changes")
        .join(vec_region_name_with::<I>(name))
}

/// Region name for a vec of type `I`, e.g. `"height/Height"`.
pub fn vec_region_name_with<I: VecIndex>(name: &str) -> String {
    vec_region_name(name, I::to_string())
}

/// Formats a vec's region name as `{name}/{index}`.
pub fn vec_region_name(name: &str, index: &str) -> String {
    format!("{name}/{index}")
}
