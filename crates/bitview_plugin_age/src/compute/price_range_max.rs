use std::{mem, time::Instant};

use bitview_primitives::CentsCompact;
use brk_types::{Cents, Height};
use tracing::debug;
use vecdb::VecIndex;

/// Sparse table of the spot price's range maxima, in compact cents (exact for every price a
/// cohort price map holds) to halve its ~19M entries at 1M heights.
#[derive(Default)]
pub struct PriceRangeMax {
    levels: Vec<Vec<CentsCompact>>,
    n: usize,
}

impl PriceRangeMax {
    pub fn extend(&mut self, prices: &[Cents]) {
        let new_n = prices.len();
        if new_n <= self.n {
            return;
        }

        let start = Instant::now();
        let old_n = self.n;
        let level_count = (usize::BITS - new_n.leading_zeros()) as usize;
        while self.levels.len() < level_count {
            self.levels.push(Vec::new());
        }

        self.levels[0].extend(
            prices[old_n..new_n]
                .iter()
                .map(|&price| CentsCompact::from(price)),
        );
        for level in 1..level_count {
            let half = 1 << (level - 1);
            let new_end = new_n + 1 - (1 << level);
            let old_end = self.levels[level].len();
            if new_end > old_end {
                let (previous, current) = self.levels.split_at_mut(level);
                let previous = &previous[level - 1];
                let current = &mut current[0];
                current.reserve(new_end - old_end);
                for index in old_end..new_end {
                    current.push(previous[index].max(previous[index + half]));
                }
            }
        }

        self.n = new_n;
        let entries: usize = self.levels.iter().map(Vec::len).sum();
        debug!(
            "PriceRangeMax extended: {} -> {} heights ({} new), {} levels, {:.2}MB, {:.2}ms",
            old_n,
            new_n,
            new_n - old_n,
            level_count,
            (entries * mem::size_of::<CentsCompact>()) as f64 / 1_000_000.0,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }

    /// The highest price from `from` through `to`, both included.
    #[inline]
    pub fn range_max(&self, from: Height, to: Height) -> Cents {
        let (start, end) = (from.to_usize(), to.to_usize());
        debug_assert!(start <= end && end < self.n);
        let len = end - start + 1;
        let level = (usize::BITS - len.leading_zeros() - 1) as usize;
        let width = 1 << level;
        let values = &self.levels[level];
        unsafe {
            let first = *values.get_unchecked(start);
            let last = *values.get_unchecked(end + 1 - width);
            Cents::from(first.max(last))
        }
    }
}
