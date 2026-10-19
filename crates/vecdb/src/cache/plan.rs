use std::ops::Range;

use super::{Request, Value};

pub(super) fn first_ending_after<T: Clone>(blocks: &[(usize, Value<T>)], from: usize) -> usize {
    blocks.partition_point(|(start, values)| *start + values.len() <= from)
}

/// The span holding `index`, with its start.
pub(super) fn containing<T: Clone>(
    blocks: &[(usize, Value<T>)],
    index: usize,
) -> Option<(usize, &Value<T>)> {
    let at = blocks
        .partition_point(|(start, _)| *start <= index)
        .checked_sub(1)?;
    let (start, values) = &blocks[at];
    (index - start < values.len()).then_some((*start, values))
}

pub(super) fn missing<T: Clone>(
    request: Request<'_>,
    cached: &[(usize, Value<T>)],
) -> Vec<Range<usize>> {
    let mut missing = Vec::new();
    for range in request.ranges() {
        let mut at = range.start;
        for (start, values) in &cached[first_ending_after(cached, at)..] {
            if *start >= range.end {
                break;
            }
            if at < *start {
                missing.push(at..*start)
            }
            at = at.max(*start + values.len()).min(range.end);
        }
        if at < range.end {
            missing.push(at..range.end)
        }
    }
    missing
}

pub(super) fn copy<T: Clone>(
    request: Request<'_>,
    cached: &[(usize, Value<T>)],
    loaded: &[(usize, Value<T>)],
    out: &mut Vec<T>,
) {
    match request {
        Request::Range(from, to) => {
            let mut at = from;
            while at < to {
                let (start, values) = containing(loaded, at)
                    .or_else(|| containing(cached, at))
                    .expect("source loader must cover every cache miss");
                let end = (start + values.len()).min(to);
                out.extend_from_slice(&values.slice()[at - start..end - start]);
                at = end;
            }
        }
        Request::Sorted(indices) => {
            out.reserve(indices.len());
            for &index in indices {
                let (start, values) = containing(loaded, index)
                    .or_else(|| containing(cached, index))
                    .expect("source loader must cover every cache miss");
                out.push(values.get(index - start).unwrap());
            }
        }
    }
}
