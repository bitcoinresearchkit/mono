use std::{iter, ops::Range};

use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Stamp};

/// Rows a long computation pushes between flushes, bounding the work a soft quit loses.
pub const WRITE_INTERVAL: usize = 10_000;

/// What a computation reads: its own version combined with every input's, and the rows all
/// inputs cover.
#[derive(Clone, Copy)]
pub struct Inputs {
    pub version: Version,
    pub len: usize,
}

impl Inputs {
    /// `version` combined with each input's version, in order.
    pub fn new<'a>(version: Version, inputs: impl IntoIterator<Item = &'a dyn AnyVec>) -> Self {
        let mut len = usize::MAX;
        let version =
            Version::combine_all(iter::once(version).chain(inputs.into_iter().map(|input| {
                len = len.min(input.len());
                input.version()
            })));
        Self { version, len }
    }

    /// [`prepare_computed`] against these inputs: the rows left to compute.
    pub fn prepare<'a, V: AnyStoredVec + ?Sized + 'a>(
        self,
        outputs: impl AsMut<[&'a mut V]>,
        max_from: usize,
        exit: &Exit,
    ) -> Result<Range<usize>> {
        let start = prepare_computed(outputs, self.version, max_from.min(self.len), exit)?;
        Ok(start..self.len)
    }
}

/// Resume a group at its shortest valid output, bounded by the requested rewind.
/// Every output is validated before the common rewind is persisted, including
/// updates with no new rows. Read-only views therefore observe the same prefix.
pub fn prepare_computed<'a, V: AnyStoredVec + ?Sized + 'a>(
    mut outputs: impl AsMut<[&'a mut V]>,
    version: Version,
    max_from: usize,
    exit: &Exit,
) -> Result<usize> {
    let _lock = exit.lock();
    let outputs = outputs.as_mut();
    let mut start = max_from;
    for vec in &mut *outputs {
        vec.any_validate_computed_version_or_reset(version)?;
        start = start.min(vec.len());
    }
    for vec in outputs {
        vec.any_truncate_if_needed_at(start)?;
        vec.write()?;
    }
    Ok(start)
}

/// `range` in chunks ending on multiples of [`WRITE_INTERVAL`], so a computation flushing after
/// each chunk flushes at the same heights however it resumed.
pub fn write_chunks(range: Range<usize>) -> impl Iterator<Item = Range<usize>> {
    let mut at = range.start;
    std::iter::from_fn(move || {
        (at < range.end).then(|| {
            let end = (at - at % WRITE_INTERVAL + WRITE_INTERVAL).min(range.end);
            let chunk = at..end;
            at = end;
            chunk
        })
    })
}

/// Flushes a group under the exit lock: a soft quit never persists part of it.
pub fn write_locked<'a, V: AnyStoredVec + ?Sized + 'a>(
    outputs: impl IntoIterator<Item = &'a mut V>,
    exit: &Exit,
) -> Result<()> {
    let _lock = exit.lock();
    for vec in outputs {
        vec.write()?;
    }
    Ok(())
}

/// Saves a group computed through `end` under the exit lock, stamped with its last height and
/// keeping rollback changes when asked, then flushes its database.
pub fn save_stamped<'a, V: AnyStoredVec + ?Sized + 'a>(
    db: &Database,
    outputs: impl IntoIterator<Item = &'a mut V>,
    end: usize,
    with_changes: bool,
    exit: &Exit,
) -> Result<()> {
    let _lock = exit.lock();
    let stamp = Stamp::from(Height::from(end - 1));
    for vec in outputs {
        vec.any_stamped_write_maybe_with_changes(stamp, with_changes)?;
    }
    db.flush();
    Ok(())
}
