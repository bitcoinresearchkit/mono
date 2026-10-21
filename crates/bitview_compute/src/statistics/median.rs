use brk_error::Result;

use brk_exit::Exit;
use vecdb::{
    CachePolicy, ComputeVec, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, Version,
    WritableVec,
};

use super::{Retained, window::SortedWindow};

pub trait ComputeRollingMedianFromStarts<I: VecIndex, T> {
    /// Rolling median, continuing the `retained` window when it is current. Rows before `first`
    /// have none (NaN), and no window reaches before it (dollar values: the first price).
    fn compute_rolling_median_from_starts<A>(
        &mut self,
        max_from: I,
        first: usize,
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
        first: usize,
        window_starts: &impl ReadableVec<I, I>,
        values: &impl ReadableVec<I, A>,
        retained: &mut Retained<SortedWindow>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue + Copy,
        f64: From<A>,
    {
        let version =
            window_starts.version() + values.version() + Version::TWO + Version::new(first as u32);
        self.compute_to(
            max_from,
            window_starts.len().min(values.len()),
            version,
            |this, range| {
                // (Rows before `first`: none.)
                let windowed = range.start.max(first.min(range.end));
                for _ in range.start..windowed {
                    this.push(T::from(f64::NAN));
                }
                let mut window = retained
                    .take(windowed, version)
                    .unwrap_or_else(|| SortedWindow::read(windowed, first, window_starts, values));
                let starts = window_starts.collect_range_at(windowed, range.end);
                let entering = values.collect_range_at(windowed, range.end);
                for (index, (start, value)) in
                    (windowed..range.end).zip(starts.into_iter().zip(entering))
                {
                    window.push(index, f64::from(value), start.to_usize().max(first));
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
