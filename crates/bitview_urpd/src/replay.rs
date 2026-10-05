use std::ops::Range;

use brk_error::{Error, Result};
use brk_types::{Cents, Height, Timestamp, Version};
use rayon::{current_num_threads, prelude::*};
use statedb::{Reader, SNAPSHOT_INTERVAL, State};

use crate::{OriginUrpd, ReplayInputs, replay_state::ReplayState};

/// Blocks per parallel segment: the history's snapshot interval, so each segment restores its
/// start state from a snapshot.
const SEGMENT_BLOCKS: usize = SNAPSHOT_INTERVAL;

/// Where a replay stands: the origin state and its price/age view.
type Position = (State, OriginUrpd);

/// A consumer's resumable replay. Successful updates retain their histogram;
/// changed sources, reorgs and failed callbacks force reconstruction.
#[derive(Default)]
pub struct Replay {
    state: Option<ReplayState>,
}

impl Replay {
    fn for_each(
        &mut self,
        range: Range<usize>,
        inputs: ReplayInputs<'_>,
        mut consume: impl FnMut(Height, Cents, &OriginUrpd) -> Result<()>,
    ) -> Result<()> {
        if range.is_empty() {
            return Ok(());
        }
        let cached = self.state.take();
        let version = check_sources(&range, inputs)?;
        let ReplayInputs {
            history,
            prices,
            timestamps,
        } = inputs;
        let cached = match cached {
            Some(cache)
                if cache.origins.len() == range.start
                    && cache.version == version
                    && history.matches(&cache.origins)? =>
            {
                Some(cache)
            }
            _ => None,
        };
        let price_start = if cached.is_some() { range.start } else { 0 };
        let prices = prices.collect_range_dyn(price_start, range.end);
        let mut state = if let Some(mut state) = cached {
            state
                .timestamps
                .extend(timestamps.collect_range_dyn(range.start, range.end));
            state
        } else {
            let timestamps = timestamps.collect_range_dyn(0, range.end);
            let (origins, source) = restore(history, &prices, &timestamps, range.start)?;
            ReplayState {
                origins,
                source,
                timestamps,
                version,
            }
        };
        let mut cursor = history.cursor(&mut state.origins)?;
        for index in range {
            let price = prices[index - price_start];
            state.source.extend_prices(&[price])?;
            state.source.advance(&mut cursor, &state.timestamps)?;
            consume(Height::from(index), price, &state.source)?;
        }
        drop(cursor);
        self.state = Some(state);
        Ok(())
    }

    /// Hands `compute`'s result for every block of `range` to `consume`, in block order.
    /// A range longer than a segment replays its segments in parallel, each from a restored
    /// state, a window of them at a time; a shorter one replays sequentially from the
    /// retained state.
    pub fn map<W, R: Send>(
        &mut self,
        range: Range<usize>,
        inputs: ReplayInputs<'_>,
        worker: impl Fn() -> W + Sync,
        compute: impl Fn(&mut W, Height, Cents, &OriginUrpd) -> Result<R> + Sync,
        mut consume: impl FnMut(Height, R) -> Result<()>,
    ) -> Result<()> {
        if range.len() <= SEGMENT_BLOCKS {
            let mut worker = worker();
            return self.for_each(range, inputs, |height, close, source| {
                consume(height, compute(&mut worker, height, close, source)?)
            });
        }
        self.state = None;
        let version = check_sources(&range, inputs)?;
        let ReplayInputs {
            history,
            prices,
            timestamps,
        } = inputs;
        let prices = prices.collect_range_dyn(0, range.end);
        let timestamps = timestamps.collect_range_dyn(0, range.end);
        let mut segments = Vec::new();
        let mut start = range.start;
        while start < range.end {
            let end = ((start / SEGMENT_BLOCKS + 1) * SEGMENT_BLOCKS).min(range.end);
            segments.push(start..end);
            start = end;
        }
        let mut last = None;
        for window in segments.chunks(current_num_threads()) {
            let replayed = window
                .par_iter()
                .map(|segment| {
                    replay_segment(
                        segment.clone(),
                        segment.end == range.end,
                        history,
                        &prices,
                        &timestamps,
                        &worker,
                        &compute,
                    )
                })
                .collect::<Vec<_>>();
            for (segment, replayed) in window.iter().zip(replayed) {
                let (results, state) = replayed?;
                for (index, result) in segment.clone().zip(results) {
                    consume(Height::from(index), result)?;
                }
                last = state.or(last);
            }
        }
        if let Some((origins, source)) = last {
            self.state = Some(ReplayState {
                origins,
                source,
                timestamps,
                version,
            });
        }
        Ok(())
    }
}

/// One segment's results, replayed from a restored start state, and its final state when kept.
fn replay_segment<W, R>(
    segment: Range<usize>,
    keep_state: bool,
    history: &Reader<'_>,
    prices: &[Cents],
    timestamps: &[Timestamp],
    worker: &impl Fn() -> W,
    compute: &impl Fn(&mut W, Height, Cents, &OriginUrpd) -> Result<R>,
) -> Result<(Vec<R>, Option<Position>)> {
    let (mut origins, mut source) = restore(history, prices, timestamps, segment.start)?;
    let mut worker = worker();
    let mut results = Vec::with_capacity(segment.len());
    let mut cursor = history.cursor(&mut origins)?;
    for index in segment {
        let price = prices[index];
        source.extend_prices(&[price])?;
        source.advance(&mut cursor, timestamps)?;
        results.push(compute(&mut worker, Height::from(index), price, &source)?);
    }
    drop(cursor);
    Ok((results, keep_state.then_some((origins, source))))
}

/// The sources' versions, once they cover `range`.
fn check_sources(
    range: &Range<usize>,
    inputs: ReplayInputs<'_>,
) -> Result<(Version, Version, (u64, u64))> {
    let ReplayInputs {
        history,
        prices,
        timestamps,
    } = inputs;
    if range.end > history.len() || range.end > prices.len() || range.end > timestamps.len() {
        return Err(Error::Internal("incomplete URPD replay sources"));
    }
    Ok((prices.version(), timestamps.version(), history.versions()))
}

/// The position after `start` blocks, rebuilt from the nearest snapshot: how a restart and
/// every parallel segment begin.
fn restore(
    history: &Reader<'_>,
    prices: &[Cents],
    timestamps: &[Timestamp],
    start: usize,
) -> Result<Position> {
    let origins = history.state_at(start)?;
    let source = OriginUrpd::new(&origins, &prices[..start], &timestamps[..start])?;
    Ok((origins, source))
}
