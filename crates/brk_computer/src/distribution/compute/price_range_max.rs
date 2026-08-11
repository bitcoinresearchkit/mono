use std::time::Instant;

use brk_types::{Cents, Height};
use tracing::debug;
use vecdb::VecIndex;

#[derive(Debug, Clone, Default)]
pub struct PriceRangeMax {
    levels: Vec<Vec<Cents>>,
    n: usize,
}

impl PriceRangeMax {
    pub(crate) fn extend(&mut self, prices: &[Cents]) {
        let new_n = prices.len();
        if new_n <= self.n || new_n == 0 {
            return;
        }

        let start = Instant::now();
        let old_n = self.n;
        let level_count = (usize::BITS - new_n.leading_zeros()) as usize;
        while self.levels.len() < level_count {
            self.levels.push(Vec::new());
        }

        self.levels[0].extend_from_slice(&prices[old_n..new_n]);
        for level in 1..level_count {
            let half = 1 << (level - 1);
            let new_end = new_n.saturating_add(1).saturating_sub(1 << level);
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
            (entries * std::mem::size_of::<Cents>()) as f64 / 1_000_000.0,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }

    pub(crate) fn truncate(&mut self, new_n: usize) {
        if new_n >= self.n {
            return;
        }
        if new_n == 0 {
            self.levels.clear();
            self.n = 0;
            return;
        }

        let level_count = (usize::BITS - new_n.leading_zeros()) as usize;
        self.levels.truncate(level_count);
        for level in 0..level_count {
            let valid = new_n.saturating_add(1).saturating_sub(1 << level);
            self.levels[level].truncate(valid);
        }
        self.n = new_n;
    }

    #[inline]
    pub(crate) fn range_max(&self, start: usize, end: usize) -> Cents {
        debug_assert!(start <= end && end < self.n);
        let len = end - start + 1;
        let level = (usize::BITS - len.leading_zeros() - 1) as usize;
        let width = 1 << level;
        let values = &self.levels[level];
        unsafe {
            let first = *values.get_unchecked(start);
            let last = *values.get_unchecked(end + 1 - width);
            first.max(last)
        }
    }

    #[inline]
    pub(crate) fn max_between(&self, from: Height, to: Height) -> Cents {
        self.range_max(from.to_usize(), to.to_usize())
    }
}
