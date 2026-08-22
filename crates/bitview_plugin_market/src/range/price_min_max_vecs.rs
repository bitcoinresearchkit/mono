use bitview_traversable::Traversable;
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use bitview_compute::{PerBlock, Price};

#[derive(Traversable)]
pub struct PriceMinMaxVecs<M: StorageMode = Rw> {
    /// Uses a trailing 7-day monotonic-time window.
    pub _1w: Price<PerBlock<Cents, M>>,
    /// Uses a trailing 14-day monotonic-time window.
    pub _2w: Price<PerBlock<Cents, M>>,
    /// Uses a trailing 30-day monotonic-time window.
    pub _1m: Price<PerBlock<Cents, M>>,
    /// Uses a trailing 365-day monotonic-time window.
    pub _1y: Price<PerBlock<Cents, M>>,
}
