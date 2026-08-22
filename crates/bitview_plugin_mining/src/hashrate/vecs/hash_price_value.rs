use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredF32};
use vecdb::{Rw, StorageMode};

use bitview_compute::{LazyPerBlock, PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct HashPriceValueVecs<M: StorageMode = Rw> {
    /// Reported per TH/s, where one TH/s is 10^12 hashes per second.
    pub ths: PerBlock<StoredF32, M>,
    /// Running all-time minimum of the per-TH/s series, where one TH/s is 10^12
    /// hashes per second. Zero values are excluded; returns zero until the
    /// first nonzero value exists.
    pub ths_min: PerBlock<StoredF32, M>,
    /// Reported per PH/s, where one PH/s is 10^15 hashes per second; exactly
    /// 1,000 times the corresponding per-TH/s series.
    pub phs: LazyPerBlock<StoredF32>,
    /// Running all-time minimum of the per-PH/s series, where one PH/s is 10^15
    /// hashes per second. It is exactly 1,000 times the corresponding per-TH/s
    /// minimum and returns zero until the first nonzero value exists.
    pub phs_min: LazyPerBlock<StoredF32>,
    /// Per-PH/s value at the represented block divided by its running nonzero
    /// all-time minimum, minus one. Zero marks the historical floor and positive
    /// values measure the rebound above it. Returns zero before a nonzero
    /// minimum exists.
    pub rebound: PercentPerBlock<PartsPerMillionSigned64, M>,
}
