use brk_error::Result;

use vecdb::{
    AnyStoredVec, AnyVec, EagerVec, Exit, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue,
    WritableVec,
};

use crate::DistributionStats;

use super::sliding_window::SlidingWindowSorted;

/// Compute all 7 rolling distribution stats (min, max, p10, p25, median, p75, p90)
/// in a single sorted-vec pass per window.
///
/// When computing multiple windows from the same source, pass the same
/// `&mut Option<(usize, Vec<f64>)>` cache to each call — the first call reads
/// and caches, subsequent calls reuse if their range is covered.
/// Process the largest window first (1y) so its cache covers all smaller windows.
pub fn compute_rolling_distribution_from_starts<I, T, A>(
    max_from: I,
    window_starts: &impl ReadableVec<I, I>,
    values: &impl ReadableVec<I, A>,
    outputs: DistributionStats<&mut EagerVec<PcoVec<I, T>>>,
    exit: &Exit,
    values_cache: &mut Option<(usize, Vec<f64>)>,
) -> Result<()>
where
    I: VecIndex,
    T: PcoVecValue + From<f64>,
    A: VecValue + Copy,
    f64: From<A>,
{
    let DistributionStats {
        min: min_out,
        max: max_out,
        pct10: p10_out,
        pct25: p25_out,
        median: median_out,
        pct75: p75_out,
        pct90: p90_out,
    } = outputs;
    let version = window_starts.version() + values.version();

    for v in [
        &mut *min_out,
        &mut *max_out,
        &mut *p10_out,
        &mut *p25_out,
        &mut *median_out,
        &mut *p75_out,
        &mut *p90_out,
    ] {
        v.validate_and_truncate(version, max_from)?;
    }

    let skip = [
        min_out.len(),
        max_out.len(),
        p10_out.len(),
        p25_out.len(),
        median_out.len(),
        p75_out.len(),
        p90_out.len(),
    ]
    .into_iter()
    .min()
    .unwrap();

    let end = window_starts.len().min(values.len());
    if skip >= end {
        return Ok(());
    }

    let range_start = if skip > 0 {
        window_starts.collect_one_at(skip - 1).unwrap().to_usize()
    } else {
        0
    };

    // Reuse cached values if the cache covers our range, otherwise read and cache.
    let need_read = match values_cache.as_ref() {
        Some((cached_start, cached)) => {
            range_start < *cached_start || end > *cached_start + cached.len()
        }
        None => true,
    };
    if need_read {
        let mut v = Vec::with_capacity(end - range_start);
        values.for_each_range_at(range_start, end, |a: A| v.push(f64::from(a)));
        *values_cache = Some((range_start, v));
    }
    let (cached_start, cached) = values_cache.as_ref().unwrap();
    let partial_values = &cached[(range_start - cached_start)..(end - cached_start)];

    let starts_batch = window_starts.collect_range_at(skip, end);
    let capacity = starts_batch
        .iter()
        .enumerate()
        .map(|(offset, start)| (skip + offset + 1).saturating_sub(start.to_usize()))
        .max()
        .unwrap_or_default();

    let mut window = SlidingWindowSorted::with_capacity(capacity);

    if skip > 0 {
        window.reconstruct(partial_values, range_start, skip);
    }

    for v in [
        &mut *min_out,
        &mut *max_out,
        &mut *p10_out,
        &mut *p25_out,
        &mut *median_out,
        &mut *p75_out,
        &mut *p90_out,
    ] {
        v.truncate_if_needed_at(skip)?;
    }

    for (j, start) in starts_batch.into_iter().enumerate() {
        let v = partial_values[skip + j - range_start];
        let start_usize = start.to_usize();
        window.advance(v, start_usize, partial_values, range_start);

        if window.is_empty() {
            let zero = T::from(0.0);
            for v in [
                &mut *min_out,
                &mut *max_out,
                &mut *p10_out,
                &mut *p25_out,
                &mut *median_out,
                &mut *p75_out,
                &mut *p90_out,
            ] {
                v.push(zero);
            }
        } else {
            min_out.push(T::from(window.min()));
            max_out.push(T::from(window.max()));
            let [p10, p25, p50, p75, p90] = window.percentiles(&[0.10, 0.25, 0.50, 0.75, 0.90]);
            p10_out.push(T::from(p10));
            p25_out.push(T::from(p25));
            median_out.push(T::from(p50));
            p75_out.push(T::from(p75));
            p90_out.push(T::from(p90));
        }

        if min_out.batch_limit_reached() {
            let _lock = exit.lock();
            for v in [
                &mut *min_out,
                &mut *max_out,
                &mut *p10_out,
                &mut *p25_out,
                &mut *median_out,
                &mut *p75_out,
                &mut *p90_out,
            ] {
                v.write()?;
            }
        }
    }

    // Final flush
    let _lock = exit.lock();
    for v in [
        min_out, max_out, p10_out, p25_out, median_out, p75_out, p90_out,
    ] {
        v.write()?;
    }

    Ok(())
}
