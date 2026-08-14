mod cached_date;
mod cached_first_height;

use brk_traversable::Traversable;
use brk_types::{Date, Height, Timestamp};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecIndex};

pub use self::cached_date::CachedDateVec;
pub use self::cached_first_height::CachedFirstHeightVec;

/// Resolution with a pinned, storage-free first-height lookup.
#[derive(Clone, Traversable)]
pub struct ResolutionVecs<I: VecIndex> {
    pub first_height: CachedFirstHeightVec<I>,
}

impl<I: VecIndex> ResolutionVecs<I> {
    pub(crate) fn new(mapping: &impl ReadableCloneableVec<Height, I>) -> Self {
        Self {
            first_height: CachedFirstHeightVec::new(mapping.read_only_boxed_clone()),
        }
    }

    pub(crate) fn invalidate_timestamp_caches(&self) {
        self.first_height.invalidate();
    }
}

/// Resolution with pinned, storage-free date and first-height lookups.
#[derive(Clone, Traversable)]
pub struct DatedResolutionVecs<I: VecIndex> {
    pub date: CachedDateVec<I>,
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
