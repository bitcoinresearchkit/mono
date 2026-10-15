use brk_error::Result;

use brk_exit::Exit;
use vecdb::{
    AnyStoredVec, CachePolicy, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, Version,
    WritableVec,
};

use super::{DistributionStats, Retained, window::SortedWindow};
use crate::prepare_computed;

/// Rows read per chunk.
const CHUNK: usize = 20_000;

/// Compute all 7 rolling distribution stats (min, max, p10, p25, median, p75, p90)
/// from one sorted window, continuing the `retained` window when it is current.
pub fn compute_rolling_distribution_from_starts<I, T, A, P: CachePolicy>(
    max_from: I,
    window_starts: &impl ReadableVec<I, I>,
    values: &impl ReadableVec<I, A>,
    mut outputs: DistributionStats<&mut PcoVec<I, T, P>>,
    retained: &mut Retained<SortedWindow>,
    exit: &Exit,
) -> Result<()>
where
    I: VecIndex,
    T: PcoVecValue + From<f64>,
    A: VecValue + Copy,
    f64: From<A>,
{
    let version = window_starts.version() + values.version() + Version::ONE;
    let end = window_starts.len().min(values.len());
    let skip = prepare_computed(
        outputs.as_mut_array().map(|output| &mut **output),
        version,
        max_from.to_usize().min(end),
        exit,
    )?;
    if skip >= end {
        return Ok(());
    }

    let mut window = retained
        .take(skip, version)
        .unwrap_or_else(|| SortedWindow::read(skip, window_starts, values));
    let mut starts = Vec::new();
    let mut entering = Vec::new();
    for from in (skip..end).step_by(CHUNK) {
        let to = (from + CHUNK).min(end);
        window_starts.collect_range_into_at(from, to, &mut starts);
        values.collect_range_into_at(from, to, &mut entering);
        for (index, (start, &value)) in (from..).zip(starts.iter().zip(&entering)) {
            window.push(index, f64::from(value), start.to_usize());
            let [pct10, pct25, median, pct75, pct90] =
                window.percentiles([0.10, 0.25, 0.50, 0.75, 0.90]);
            outputs.min.push(T::from(window.min()));
            outputs.max.push(T::from(window.max()));
            outputs.pct10.push(T::from(pct10));
            outputs.pct25.push(T::from(pct25));
            outputs.median.push(T::from(median));
            outputs.pct75.push(T::from(pct75));
            outputs.pct90.push(T::from(pct90));
        }
        if to == end || outputs.min.batch_limit_reached() {
            let _lock = exit.lock();
            for output in outputs.as_mut_array() {
                output.write()?;
            }
        }
    }
    retained.keep(end, version, window);
    Ok(())
}
