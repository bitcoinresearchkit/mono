use std::ops::AddAssign;

use brk_types::Height;
use vecdb::{AnyVec, PcoVecValue, ReadableVec, VecValue, WritableVec};

use crate::CachedSeries;

/// Writer-only running totals, reloaded from the source when the requested length differs from
/// the cached one.
pub struct CumulativeState<R> {
    last: Option<(usize, R)>,
}

impl<R> Default for CumulativeState<R> {
    fn default() -> Self {
        Self { last: None }
    }
}

impl<R: Clone + Default> CumulativeState<R> {
    pub fn accumulate(
        &mut self,
        len: usize,
        load: impl FnOnce() -> Option<R>,
        add: impl FnOnce(&mut R),
    ) -> R {
        let mut values = match self.last.take() {
            Some((cached_len, values)) if cached_len == len => values,
            _ => load().unwrap_or_default(),
        };
        add(&mut values);
        self.last = Some((len + 1, values.clone()));
        values
    }

    /// The running total at `height`, `value` more than the total `stored` holds below it.
    #[inline]
    pub(crate) fn add_at(
        &mut self,
        height: Height,
        stored: &impl ReadableVec<Height, R>,
        value: R,
    ) -> R
    where
        R: VecValue + AddAssign,
    {
        self.accumulate(
            usize::from(height),
            || {
                height
                    .decremented()
                    .and_then(|height| stored.collect_one(height))
            },
            |total| *total += value,
        )
    }

    /// Appends to `stored` the running total through one more block of `value`.
    #[inline(always)]
    pub(crate) fn push(&mut self, stored: &mut CachedSeries<Height, R>, value: R)
    where
        R: PcoVecValue + AddAssign,
    {
        let total = self.add_at(Height::from(stored.len()), stored, value);
        stored.push(total);
    }
}
