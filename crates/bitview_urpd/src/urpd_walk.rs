use std::ops::Range;

use brk_error::{Error, Result};
use brk_types::Height;
use rayon::{current_num_threads, prelude::*};

use crate::{UrpdCells, UrpdHistory};

/// Blocks per parallel segment: long enough that each segment's restore (a checkpoint and its
/// following changes) stays a small share of its work.
const SEGMENT_BLOCKS: usize = 5000;

/// A consumer's walk over the URPD history: the cells after each block it computes, in block
/// order. The cells after a complete walk are kept, and the next walk resumes from them when it
/// starts where they end on the same chain.
#[derive(Default)]
pub struct UrpdWalk {
    last: Option<(usize, UrpdCells)>,
}

impl UrpdWalk {
    /// Hands `compute`'s result for every block of `range` to `consume`, in block order. A range
    /// longer than a segment walks its segments in parallel, each from its own restore, a window
    /// of them at a time; a shorter one walks on from the kept cells. `worker` builds the scratch
    /// for the blocks it is given, so its cursors can read just those.
    pub fn map<W, R: Send>(
        &mut self,
        range: Range<usize>,
        history: &UrpdHistory,
        worker: impl Fn(Range<usize>) -> W + Sync,
        compute: impl Fn(&mut W, Height, &UrpdCells) -> Result<R> + Sync,
        mut consume: impl FnMut(Height, R) -> Result<()>,
    ) -> Result<()> {
        let last = self.last.take();
        if range.end > history.len() {
            return Err(Error::Internal("URPD history short of the walk"));
        }
        if range.is_empty() {
            self.last = last;
            return Ok(());
        }
        if range.len() <= SEGMENT_BLOCKS {
            let mut cells = match last {
                Some((end, cells)) if end == range.start && history.follows(&cells, end)? => cells,
                _ => history.at(range.start, range.end)?,
            };
            let mut worker = worker(range.clone());
            history.walk(range.clone(), &mut cells, |height, cells| {
                consume(height, compute(&mut worker, height, cells)?)
            })?;
            self.last = Some((range.end, cells));
            return Ok(());
        }
        let mut segments = Vec::new();
        let mut start = range.start;
        while start < range.end {
            let end = ((start / SEGMENT_BLOCKS + 1) * SEGMENT_BLOCKS).min(range.end);
            segments.push(start..end);
            start = end;
        }
        let mut last = None;
        for window in segments.chunks(current_num_threads()) {
            let walked = window
                .par_iter()
                .map(|segment| -> Result<_> {
                    let mut cells = history.at(segment.start, range.end)?;
                    let mut worker = worker(segment.clone());
                    let mut results = Vec::with_capacity(segment.len());
                    history.walk(segment.clone(), &mut cells, |height, cells| {
                        results.push(compute(&mut worker, height, cells)?);
                        Ok(())
                    })?;
                    Ok((results, (segment.end == range.end).then_some(cells)))
                })
                .collect::<Vec<_>>();
            for (segment, walked) in window.iter().zip(walked) {
                let (results, cells) = walked?;
                for (height, result) in segment.clone().zip(results) {
                    consume(Height::from(height), result)?;
                }
                last = cells.or(last);
            }
        }
        self.last = last.map(|cells| (range.end, cells));
        Ok(())
    }
}
