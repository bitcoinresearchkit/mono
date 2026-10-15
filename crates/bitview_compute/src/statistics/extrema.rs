use std::collections::VecDeque;

use brk_error::Result;
use brk_exit::Exit;
use vecdb::{ReadableVec, StoredVec, VecIndex, VecValue};

use super::Retained;
use crate::prepare_computed;

/// Rows read per chunk.
const CHUNK: usize = 20_000;

/// The monotonic deques of a rolling minimum and maximum: the candidates `(row, value)` that
/// can still become the window's extreme.
pub struct ExtremaWindow<A> {
    lower: VecDeque<(usize, A)>,
    upper: VecDeque<(usize, A)>,
}

impl<A: Copy + Ord> ExtremaWindow<A> {
    /// Rebuilds the window of the row before `next` from history.
    fn read<I: VecIndex>(
        next: usize,
        starts: &impl ReadableVec<I, I>,
        source: &impl ReadableVec<I, A>,
    ) -> Self
    where
        A: VecValue,
    {
        let mut window = Self {
            lower: VecDeque::new(),
            upper: VecDeque::new(),
        };
        if let Some(last) = next.checked_sub(1) {
            let mut index = starts.collect_one_at(last).unwrap().to_usize();
            source.for_each_range_at(index, next, |value| {
                window.enter(index, value);
                index += 1;
            });
        }
        window
    }

    #[inline]
    fn enter(&mut self, index: usize, value: A) {
        push::<A, false>(&mut self.lower, index, value);
        push::<A, true>(&mut self.upper, index, value);
    }

    /// Enters row `index`'s value and drops the rows before its window `start`.
    #[inline]
    fn push(&mut self, index: usize, value: A, start: usize) -> (A, A) {
        while self.lower.front().is_some_and(|(index, _)| *index < start) {
            self.lower.pop_front();
        }
        while self.upper.front().is_some_and(|(index, _)| *index < start) {
            self.upper.pop_front();
        }
        self.enter(index, value);
        (self.lower.front().unwrap().1, self.upper.front().unwrap().1)
    }
}

/// Compute a rolling minimum and maximum with one source read, continuing the `retained`
/// window when it is current.
pub fn compute_rolling_extrema_from_starts<V, A>(
    min: &mut V,
    max: &mut V,
    max_from: V::I,
    starts: &impl ReadableVec<V::I, V::I>,
    source: &impl ReadableVec<V::I, A>,
    retained: &mut Retained<ExtremaWindow<A>>,
    exit: &Exit,
) -> Result<()>
where
    V: StoredVec,
    A: VecValue + Copy + Ord,
    V::T: From<A>,
{
    let version = starts.version() + source.version();
    let end = starts.len().min(source.len());
    let from = prepare_computed(
        [&mut *min, &mut *max],
        version,
        max_from.to_usize().min(end),
        exit,
    )?;
    if from >= end {
        return Ok(());
    }
    let mut window = retained
        .take(from, version)
        .unwrap_or_else(|| ExtremaWindow::read(from, starts, source));
    let mut boundaries = Vec::new();
    let mut values = Vec::new();
    for from in (from..end).step_by(CHUNK) {
        let to = (from + CHUNK).min(end);
        starts.collect_range_into_at(from, to, &mut boundaries);
        source.collect_range_into_at(from, to, &mut values);
        for (index, (start, &value)) in (from..).zip(boundaries.iter().zip(&values)) {
            let (lower, upper) = window.push(index, value, start.to_usize());
            min.push(V::T::from(lower));
            max.push(V::T::from(upper));
        }
        let _lock = exit.lock();
        min.write()?;
        max.write()?;
    }
    retained.keep(end, version, window);
    Ok(())
}

#[inline]
fn push<A: Copy + Ord, const MAX: bool>(deque: &mut VecDeque<(usize, A)>, index: usize, value: A) {
    while deque
        .back()
        .is_some_and(|(_, old)| if MAX { *old <= value } else { *old >= value })
    {
        deque.pop_back();
    }
    deque.push_back((index, value));
}
