use brk_traversable::Traversable;
use brk_types::{Date, Height, Timestamp};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecIndex};

use super::{CachedDateVec, CachedFirstHeightVec};

/// Resolution with pinned, storage-free date and first-height lookups.
#[derive(Clone, Traversable)]
pub struct DatedResolutionVecs<I: VecIndex> {
    /// UTC calendar date in `YYYY-MM-DD` format associated with the time-period
    /// index. At `day1`, this is the represented calendar day. At coarser
    /// indexes, it is derived from the first monotonic block timestamp at or
    /// after the period.
    pub date: CachedDateVec<I>,
    /// Lowest block height whose resolution index is at least the requested
    /// index. Empty indexes therefore use the first height of the next populated
    /// index; indexes preceding the first populated one resolve to height 0.
    pub first_height: CachedFirstHeightVec<I>,
}

impl<I: VecIndex> DatedResolutionVecs<I> {
    pub(crate) fn from_period_date(
        mapping: &impl ReadableCloneableVec<Height, I>,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        period_from_timestamp: fn(Timestamp) -> I,
    ) -> Self
    where
        Date: From<I>,
    {
        Self::new(mapping, timestamps, period_from_timestamp, |period, _| {
            Date::from(period)
        })
    }

    pub(crate) fn from_first_timestamp(
        mapping: &impl ReadableCloneableVec<Height, I>,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        period_from_timestamp: fn(Timestamp) -> I,
    ) -> Self {
        Self::new(
            mapping,
            timestamps,
            period_from_timestamp,
            |_, timestamp| Date::from(timestamp),
        )
    }

    fn new(
        mapping: &impl ReadableCloneableVec<Height, I>,
        timestamps: ReadableBoxedVec<Height, Timestamp>,
        period_from_timestamp: fn(Timestamp) -> I,
        date_from_period_and_timestamp: fn(I, Timestamp) -> Date,
    ) -> Self {
        Self {
            date: CachedDateVec::new(
                timestamps,
                period_from_timestamp,
                date_from_period_and_timestamp,
            ),
            first_height: CachedFirstHeightVec::new(mapping.read_only_boxed_clone()),
        }
    }

    pub(crate) fn invalidate_timestamp_caches(&self) {
        self.date.invalidate();
        self.first_height.invalidate();
    }
}
