use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{ErrorKind, Write},
    mem,
    path::{Path, PathBuf},
};

use super::{ChangeCursor, ChangeData, ReadWriteBaseVec, changes_path_with};
use crate::{Bytes, Error, Result, SIZE_OF_U64, Stamp, ValueStrategy, VecIndex, VecValue};

impl<I, T> ReadWriteBaseVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    pub fn changes_path(&self) -> PathBuf {
        changes_path_with::<I>(&self.db(), &self.name)
    }

    pub fn serialize_changes<S: ValueStrategy<T>>(
        &self,
        collect_stored: impl FnOnce(usize, usize) -> Result<Vec<T>>,
    ) -> Result<Vec<u8>> {
        let write_values = |values: &[T], bytes: &mut Vec<u8>| {
            for value in values {
                S::write_to_vec(value, bytes);
            }
        };
        let prev_stored_len = self.prev_stored_len();
        let stored_len = self.stored_len();
        let truncated = prev_stored_len.saturating_sub(stored_len);

        let value_count = truncated + self.prev_pushed().len();
        let mut bytes = Vec::with_capacity(6 * SIZE_OF_U64 + value_count * size_of::<T>());

        bytes.extend(self.header.stamp().to_bytes());
        bytes.extend(prev_stored_len.to_bytes());
        bytes.extend(stored_len.to_bytes());
        bytes.extend(truncated.to_bytes());

        if truncated > 0 {
            let truncated_vals = collect_stored(stored_len, prev_stored_len)?;
            write_values(&truncated_vals, &mut bytes);
        }

        bytes.extend(self.prev_pushed().len().to_bytes());
        write_values(self.prev_pushed(), &mut bytes);

        // Rollback drops newly appended values by restoring the previous length.
        bytes.extend(0_usize.to_bytes());

        Ok(bytes)
    }

    /// Returns `Error::Overflow` on arithmetic overflow,
    /// `Error::WrongLength` if the data is truncated.
    pub fn parse_change_data<S: ValueStrategy<T>>(c: &mut ChangeCursor) -> Result<ChangeData<T>> {
        let size_of_t = size_of::<T>();
        let prev_stamp = c.read_stamp()?;
        let prev_stored_len = c.read_u64()?;
        c.skip(SIZE_OF_U64)?; // stored_len, not needed for rollback
        let truncated_count = c.read_u64()?;

        let truncated_start = prev_stored_len
            .checked_sub(truncated_count)
            .ok_or(Error::Underflow)?;
        let truncated_values = c.read_values(truncated_count, size_of_t, S::read)?;

        let prev_pushed_len = c.read_u64()?;
        let prev_pushed = c.read_values(prev_pushed_len, size_of_t, S::read)?;

        let pushed_len = c.read_u64()?;
        c.skip(size_of_t.checked_mul(pushed_len).ok_or(Error::Overflow)?)?;

        Ok(ChangeData {
            prev_stamp,
            truncated_start,
            truncated_values,
            prev_pushed,
        })
    }

    /// Persisted length after undoing `change` from the last save point.
    fn rollback_stored_len(&self, change: &ChangeData<T>) -> usize {
        change.truncated_start.min(self.previous_stored_len)
    }

    /// First row a rollback with `change` replaces: the rows below stay as they are.
    pub fn rollback_from(&self, change: &ChangeData<T>) -> usize {
        self.rollback_stored_len(change).min(self.stored_len())
    }

    /// Undoes one change record from the last save point: the state its stamp wrote, or what an
    /// earlier undo of this rollback restored (partly in memory). Unsaved changes are dropped:
    /// rows an unflushed truncation cut stay on disk until the save point says otherwise.
    /// Restored rows are kept and trimmed like persisted ones, so undo records compose until
    /// the next write persists the result.
    pub fn apply_rollback(&mut self, change: ChangeData<T>) {
        let stored_len = self.rollback_stored_len(&change);
        let mut pushed = mem::take(self.pushed.previous_mut());
        debug_assert!(
            self.previous_stored_len + pushed.len() >= change.truncated_start,
            "the state to undo is shorter than its change record's unchanged prefix"
        );
        // Rows in [stored_len, truncated_start) are unchanged but not on disk: they stay pushed.
        pushed.truncate(change.truncated_start - stored_len);
        pushed.extend(change.truncated_values);
        pushed.extend(change.prev_pushed);
        self.read_only.header.update_stamp(change.prev_stamp);
        self.read_only.stored_len.set(stored_len);
        self.previous_stored_len = stored_len;
        *self.pushed.current_mut() = pushed;
        self.pushed.save();
        self.unwritten_baseline = true;
    }

    /// Caller must check `saved_stamped_changes > 0` before calling.
    pub fn save_change_file(&self, stamp: Stamp, data: &[u8]) -> Result<()> {
        debug_assert!(self.saved_stamped_changes > 0);
        // A record at the current stamp would replace that stamp's own undo with a self-link.
        debug_assert!(
            stamp > self.header.stamp(),
            "{}: undo record at stamp {stamp:?} does not follow the vec's stamp {:?}",
            self.name,
            self.header.stamp()
        );
        let path = self.changes_path();
        let mut cached = self.change_stamps.lock();
        // Taken while files change: after an error the next call lists the directory again.
        let mut stamps = match cached.take() {
            Some(stamps) => stamps,
            None => {
                fs::create_dir_all(&path)?;
                list_change_stamps(&path)?
            }
        };

        // Never truncate an existing undo record until its replacement is
        // complete. Ignore this non-numeric staging name when listing stamps.
        let pending = path.join("pending");
        File::create(&pending)?.write_all(data)?;
        fs::rename(&pending, change_file(&path, stamp))?;
        if let Err(at) = stamps.binary_search(&stamp) {
            stamps.insert(at, stamp);
        }

        // Prune only after the new undo record is in place: keep `stamp` and the newest
        // `saved_stamped_changes - 1` older records.
        let at = stamps.partition_point(|&saved| saved < stamp);
        let excess = at.saturating_sub(self.saved_stamped_changes as usize - 1);
        for &saved in stamps[..excess].iter().chain(&stamps[at + 1..]) {
            fs::remove_file(change_file(&path, saved))?;
        }
        stamps.truncate(at + 1);
        stamps.drain(..excess);
        *cached = Some(stamps);
        Ok(())
    }

    /// Removes the change files above `stamp`, the vec's stamp before a write without undo: left
    /// by an earlier rollback (kept until the vec writes again), they undo states the write
    /// replaces, so a later rollback must stop at it instead of walking through them.
    pub fn remove_changes_above(&self, stamp: Stamp) -> Result<()> {
        if self.saved_stamped_changes == 0 {
            return Ok(());
        }
        let path = self.changes_path();
        let mut cached = self.change_stamps.lock();
        let mut stamps = match cached.take() {
            Some(stamps) => stamps,
            None => match list_existing(&path)? {
                Some(stamps) => stamps,
                None => return Ok(()),
            },
        };
        let at = stamps.partition_point(|&saved| saved <= stamp);
        for &saved in &stamps[at..] {
            fs::remove_file(change_file(&path, saved))?;
        }
        stamps.truncate(at);
        *cached = Some(stamps);
        Ok(())
    }

    /// Forgets the retained change-file inventory: the directory was removed.
    pub(crate) fn forget_change_files(&self) {
        *self.change_stamps.lock() = None;
    }

    pub fn save_prev(&mut self) {
        self.previous_stored_len = self.stored_len();
        self.pushed.previous_mut().clear();
    }

    pub fn save_prev_for_rollback(&mut self) {
        self.previous_stored_len = self.stored_len();
        self.pushed.save();
    }

    pub fn read_current_change_file(&self) -> Result<Vec<u8>> {
        Ok(fs::read(change_file(
            &self.changes_path(),
            self.header.stamp(),
        ))?)
    }

    /// The change files by stamp: none when the vec never wrote one (no changes directory).
    pub fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>> {
        let path = self.changes_path();
        let mut cached = self.change_stamps.lock();
        if cached.is_none() {
            *cached = list_existing(&path)?;
        }
        Ok(cached
            .iter()
            .flatten()
            .map(|&stamp| (stamp, change_file(&path, stamp)))
            .collect())
    }
}

fn change_file(path: &Path, stamp: Stamp) -> PathBuf {
    path.join(u64::from(stamp).to_string())
}

/// The sorted change-file stamps, or `None` without a changes directory.
fn list_existing(path: &Path) -> Result<Option<Vec<Stamp>>> {
    match list_change_stamps(path) {
        Ok(stamps) => Ok(Some(stamps)),
        Err(Error::IO(err)) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

fn list_change_stamps(path: &Path) -> Result<Vec<Stamp>> {
    let mut stamps = fs::read_dir(path)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            Some(Stamp::from(
                path.file_name()?.to_str()?.parse::<u64>().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    stamps.sort_unstable();
    Ok(stamps)
}
