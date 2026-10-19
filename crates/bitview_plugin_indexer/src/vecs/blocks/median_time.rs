use brk_types::Timestamp;
use vecdb::{AnyVec, ReadableVec, WritableVec};

use super::BlocksVecs;

/// The last eleven timestamps, by height modulo eleven, and the height after the newest:
/// the next run continuing there reads no earlier timestamp back.
#[derive(Default)]
pub struct MedianTimeWindow {
    next: Option<usize>,
    timestamps: [Timestamp; 11],
}

impl BlocksVecs {
    /// Extend once per export batch, continuing the retained window or seeding it from at most
    /// ten preceding timestamps. The source cursor decodes each page once; neither the source
    /// history nor a per-block heap buffer is materialized.
    pub fn compute_median_times(&mut self) {
        let begin = self.median_time.len();
        let end = self.timestamp.len();
        debug_assert!(begin <= end, "Median time exceeds timestamp length");
        if begin == end {
            return;
        }
        let window = &mut self.median_window;
        let mut height = if window.next == Some(begin) {
            begin
        } else {
            begin.saturating_sub(10)
        };
        self.timestamp.for_each_range_at(height, end, |timestamp| {
            window.timestamps[height % 11] = timestamp;
            if height >= begin {
                let len = (height + 1).min(11);
                let mut sorted = window.timestamps;
                sorted[..len].sort_unstable();
                self.median_time.push(sorted[len / 2]);
            }
            height += 1;
        });
        debug_assert_eq!(self.median_time.len(), end, "Incomplete median time source");
        self.median_window.next = Some(end);
    }
}
