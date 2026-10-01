use std::collections::VecDeque;

use brk_error::Result;
use brk_exit::Exit;
use vecdb::{AnyStoredVec, EagerVec, ReadableVec, StoredVec, VecIndex, VecValue, WritableVec};

use crate::prepare_computed;

/// Compute a rolling minimum and maximum with one source read and one warmup.
pub fn compute_rolling_extrema_from_starts<V, A>(
    min: &mut EagerVec<V>,
    max: &mut EagerVec<V>,
    max_from: V::I,
    starts: &impl ReadableVec<V::I, V::I>,
    source: &impl ReadableVec<V::I, A>,
    exit: &Exit,
) -> Result<()>
where
    V: StoredVec,
    A: VecValue + Copy + Ord,
    V::T: From<A>,
{
    let end = starts.len().min(source.len());
    let from = prepare_computed(
        [&mut *min, &mut *max],
        starts.version() + source.version(),
        max_from.to_usize().min(end),
        exit,
    )?;
    let mut lower = VecDeque::new();
    let mut upper = VecDeque::new();
    if from > 0 && from < end {
        let warmup = starts.collect_one_at(from - 1).unwrap().to_usize();
        let mut index = warmup;
        source.for_each_range_at(warmup, from, |value| {
            push::<A, false>(&mut lower, index, value);
            push::<A, true>(&mut upper, index, value);
            index += 1;
        });
    }
    let mut boundaries = Vec::new();
    let mut values = Vec::new();
    for from in (from..end).step_by(20_000) {
        let to = (from + 20_000).min(end);
        starts.collect_range_into_at(from, to, &mut boundaries);
        source.collect_range_into_at(from, to, &mut values);
        for (offset, (&start, &value)) in boundaries.iter().zip(&values).enumerate() {
            let index = from + offset;
            let start = start.to_usize();
            while lower.front().is_some_and(|(index, _)| *index < start) {
                lower.pop_front();
            }
            while upper.front().is_some_and(|(index, _)| *index < start) {
                upper.pop_front();
            }
            push::<A, false>(&mut lower, index, value);
            push::<A, true>(&mut upper, index, value);
            min.push(V::T::from(lower.front().unwrap().1));
            max.push(V::T::from(upper.front().unwrap().1));
        }
        let _lock = exit.lock();
        min.write()?;
        max.write()?;
    }
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
