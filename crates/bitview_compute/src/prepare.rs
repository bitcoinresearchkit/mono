use std::{iter, ops::Range};

use brk_error::Result;
use brk_exit::Exit;
use brk_types::Version;
use vecdb::{AnyStoredVec, AnyVec};

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
        let version = iter::once(version)
            .chain(inputs.into_iter().map(|input| {
                len = len.min(input.len());
                input.version()
            }))
            .sum::<Version>();
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

/// Blocks at the end of every update written with undo: a reorg removing at most this many
/// blocks rolls back to at or before its start, however long the update was.
const UNDO_TAIL: usize = 10;

/// Where the protected tail of an update over `range` starts: its last [`UNDO_TAIL`] blocks.
pub fn undo_tail_start(range: &Range<usize>) -> usize {
    range.end.saturating_sub(UNDO_TAIL).max(range.start)
}

/// `range` in the writes of a computation that keeps undo, each with whether it does:
/// [`WRITE_INTERVAL`]-block chunks from the start (the bulk, whose last write is the baseline the
/// undo lands on), then the protected tail ([`undo_tail_start`]), the only write with undo
/// unless it is genesis alone: that is the initial baseline, at a fresh vector's stamp, and no
/// rollback reaches before it (its undo would link the stamp to itself). An update of at most
/// [`UNDO_TAIL`] blocks is that one write.
pub fn undo_chunks(range: Range<usize>) -> impl Iterator<Item = (Range<usize>, bool)> {
    let tail = undo_tail_start(&range);
    (range.start..tail)
        .step_by(WRITE_INTERVAL)
        .map(move |from| (from..(from + WRITE_INTERVAL).min(tail), false))
        .chain((tail < range.end).then_some((tail..range.end, range.end > 1)))
}

/// Writes a group of appended outputs under the exit lock: a soft quit never persists part of
/// it. They resume by length and version ([`prepare_computed`]), so they carry no stamp; only a
/// store rewritten in place writes one, with its undo.
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
