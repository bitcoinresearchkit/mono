use brk_error::Result;

use brk_exit::Exit;
use vecdb::{
    CachePolicy, ComputeVec, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, Version,
    WritableVec,
};

use super::{Retained, window::SortedWindow};

pub trait ComputeRollingMedianFromStarts<I: VecIndex, T> {
    /// Rolling median, continuing the `retained` window when it is current.
    fn compute_rolling_median_from_starts<A>(
        &mut self,
        max_from: I,
        window_starts: &impl ReadableVec<I, I>,
        values: &impl ReadableVec<I, A>,
        retained: &mut Retained<SortedWindow>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue + Copy,
        f64: From<A>;
}

impl<I, T, P: CachePolicy> ComputeRollingMedianFromStarts<I, T> for PcoVec<I, T, P>
where
    I: VecIndex,
    T: PcoVecValue + From<f64>,
{
    fn compute_rolling_median_from_starts<A>(
        &mut self,
        max_from: I,
        window_starts: &impl ReadableVec<I, I>,
        values: &impl ReadableVec<I, A>,
        retained: &mut Retained<SortedWindow>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue + Copy,
        f64: From<A>,
    {
        let version = window_starts.version() + values.version() + Version::ONE;
        self.compute_to(
            max_from,
            window_starts.len().min(values.len()),
            version,
            |this, range| {
                let mut window = retained
                    .take(range.start, version)
                    .unwrap_or_else(|| SortedWindow::read(range.start, window_starts, values));
                let starts = window_starts.collect_range_at(range.start, range.end);
                let entering = values.collect_range_at(range.start, range.end);
                for (index, (start, value)) in range.clone().zip(starts.into_iter().zip(entering)) {
                    window.push(index, f64::from(value), start.to_usize());
                    let [median] = window.percentiles([0.50]);
                    this.push(T::from(median));
                }
                retained.keep(range.end, version, window);
                Ok(())
            },
            exit,
        )?;
        Ok(())
    }
}
