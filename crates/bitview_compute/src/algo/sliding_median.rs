use brk_error::Result;

use brk_exit::Exit;
use vecdb::{AnyVec, EagerVec, PcoVec, PcoVecValue, ReadableVec, VecIndex, VecValue, WritableVec};

use super::sliding_window::SlidingWindowSorted;

pub trait ComputeRollingMedianFromStarts<I: VecIndex, T> {
    fn compute_rolling_median_from_starts<A>(
        &mut self,
        max_from: I,
        window_starts: &impl ReadableVec<I, I>,
        values: &impl ReadableVec<I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue + Copy,
        f64: From<A>;
}

impl<I, T> ComputeRollingMedianFromStarts<I, T> for EagerVec<PcoVec<I, T>>
where
    I: VecIndex,
    T: PcoVecValue + From<f64>,
{
    fn compute_rolling_median_from_starts<A>(
        &mut self,
        max_from: I,
        window_starts: &impl ReadableVec<I, I>,
        values: &impl ReadableVec<I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue + Copy,
        f64: From<A>,
    {
        self.validate_and_truncate(window_starts.version() + values.version(), max_from)?;

        self.repeat_until_complete(exit, |this| {
            let skip = this.len();
            let end = window_starts.len().min(values.len());

            let range_start = if skip > 0 {
                window_starts.collect_one_at(skip - 1).unwrap().to_usize()
            } else {
                0
            };
            let mut partial_values: Vec<f64> = Vec::with_capacity(end - range_start);
            values.for_each_range_at(range_start, end, |a: A| partial_values.push(f64::from(a)));

            let capacity = if skip > 0 && skip < end {
                let first_start = window_starts.collect_one_at(skip).unwrap().to_usize();
                (skip + 1).saturating_sub(first_start)
            } else if !partial_values.is_empty() {
                partial_values.len().min(1024)
            } else {
                0
            };

            let mut window = SlidingWindowSorted::with_capacity(capacity);

            if skip > 0 {
                window.reconstruct(&partial_values, range_start, skip);
            }

            let starts_batch = window_starts.collect_range_at(skip, end);

            for (j, start) in starts_batch.into_iter().enumerate() {
                let i = skip + j;
                let v = partial_values[i - range_start];
                let start_usize = start.to_usize();
                window.advance(v, start_usize, &partial_values, range_start);

                let median = window.percentile(0.50);
                this.push(T::from(median));

                if this.batch_limit_reached() {
                    break;
                }
            }

            Ok(())
        })?;

        Ok(())
    }
}
