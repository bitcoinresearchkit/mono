use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::{AnyVec, CachePolicy, EagerVec, PcoVec, ReadableVec, VecIndex, VecValue, WritableVec};

use crate::NumericValue;

#[allow(clippy::too_many_arguments)]
pub fn compute_cumulative_sum_from_indexes<A, B, S, T, P: CachePolicy>(
    target: &mut EagerVec<PcoVec<Height, T, P>>,
    max_from: Height,
    first_indexes: &impl ReadableVec<Height, A>,
    indexes_count: &impl ReadableVec<Height, B>,
    source: &impl ReadableVec<A, S>,
    mut transform: impl FnMut(S) -> T,
    exit: &Exit,
) -> Result<()>
where
    A: VecIndex + VecValue,
    B: VecValue,
    S: VecValue,
    T: NumericValue,
    usize: From<B>,
{
    target.validate_computed_version_or_reset(
        first_indexes.version() + indexes_count.version() + source.version(),
    )?;
    target.truncate_if_needed(max_from)?;
    target.repeat_until_complete(exit, |target| {
        let skip = target.len();
        let end = target
            .batch_end(indexes_count.len())
            .min(first_indexes.len());
        if skip >= end {
            return Ok(());
        }

        let source_start = first_indexes.collect_one_at(skip).unwrap().to_usize();
        let counts: Vec<usize> = indexes_count
            .collect_range_at(skip, end)
            .into_iter()
            .map(usize::from)
            .collect();
        let source_end = source_start + counts.iter().sum::<usize>();
        let mut cumulative = skip
            .checked_sub(1)
            .and_then(|index| target.collect_one_at(index))
            .unwrap_or_default();
        let mut group_index = 0;

        while group_index < counts.len() && counts[group_index] == 0 {
            target.push(cumulative);
            group_index += 1;
        }

        if group_index < counts.len() {
            let mut remaining = counts[group_index];
            source.fold_range_at(source_start, source_end, T::default(), |sum, value| {
                let sum = sum + transform(value);
                remaining -= 1;
                if remaining == 0 {
                    cumulative += sum;
                    target.push(cumulative);
                    group_index += 1;
                    while group_index < counts.len() && counts[group_index] == 0 {
                        target.push(cumulative);
                        group_index += 1;
                    }
                    if group_index < counts.len() {
                        remaining = counts[group_index];
                    }
                    T::default()
                } else {
                    sum
                }
            });
        }

        Ok(())
    })?;
    Ok(())
}
