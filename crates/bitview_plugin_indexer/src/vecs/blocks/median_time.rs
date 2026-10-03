use brk_error::{Error, Result};
use brk_types::Timestamp;
use vecdb::{AnyVec, ReadableVec, WritableVec};

use super::BlocksVecs;

impl BlocksVecs {
    /// Extend once per export batch, seeding the rolling window from at most
    /// ten preceding timestamps. The source cursor decodes each page once;
    /// neither the source history nor a per-block heap buffer is materialized.
    pub fn compute_median_times(&mut self) -> Result<()> {
        let begin = self.median_time.len();
        let end = self.timestamp.len();
        if begin > end {
            return Err(Error::Internal("Median time exceeds timestamp length"));
        }
        if begin == end {
            return Ok(());
        }
        let mut window = [Timestamp::ZERO; 11];
        let mut height = begin.saturating_sub(10);
        self.timestamp.for_each_range_at(height, end, |timestamp| {
            window[height % 11] = timestamp;
            if height >= begin {
                let len = (height + 1).min(11);
                let mut sorted = window;
                sorted[..len].sort_unstable();
                self.median_time.push(sorted[len / 2]);
            }
            height += 1;
        });
        if self.median_time.len() != end {
            return Err(Error::Internal("Incomplete median time source"));
        }
        Ok(())
    }
}
