use bitview_primitives::Date;
use bitview_traversable::Traversable;
use bitview_vecs::RangeMapVec;
use brk_types::{Height, Timestamp};
use derive_more::{Deref, DerefMut};
use rangeindex::SharedRangeMap;
use vecdb::{AnyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, VecIndex};

use super::ResolutionVecs;

#[derive(Clone)]
enum DateSource<I> {
    Period(fn(I) -> Date),
    FirstTimestamp(ReadableBoxedVec<Height, Timestamp>),
}

/// Resident date and first-height lookups, updated together before publication.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct DatedResolutionVecs<I: VecIndex> {
    /// UTC calendar date in `YYYY-MM-DD` format associated with the time-period
    /// index. At `day1`, this is the represented calendar day. At coarser
    /// indexes, it is derived from the first monotonic block timestamp at or
    /// after the period.
    pub date: RangeMapVec<I, Date>,
    #[traversable(skip)]
    dates: SharedRangeMap<Date, I>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    resolution: ResolutionVecs<I>,
    #[traversable(skip)]
    date_source: DateSource<I>,
}

impl<I: VecIndex> DatedResolutionVecs<I> {
    pub fn from_period_date(mapping: &impl ReadableCloneableVec<Height, I>) -> Self
    where
        Date: From<I>,
    {
        Self::new(mapping, DateSource::Period(Date::from))
    }

    pub fn from_first_timestamp(
        mapping: &impl ReadableCloneableVec<Height, I>,
        timestamps: &impl ReadableCloneableVec<Height, Timestamp>,
    ) -> Self {
        Self::new(
            mapping,
            DateSource::FirstTimestamp(timestamps.read_only_boxed_clone()),
        )
    }

    fn new(mapping: &impl ReadableCloneableVec<Height, I>, date_source: DateSource<I>) -> Self {
        let version = match &date_source {
            DateSource::Period(_) => mapping.version(),
            DateSource::FirstTimestamp(timestamps) => timestamps.version(),
        };
        let dates = SharedRangeMap::new(Vec::new());
        let this = Self {
            date: RangeMapVec::new("date", version, dates.clone()),
            dates,
            resolution: ResolutionVecs::new(mapping),
            date_source,
        };
        this.update_dates(0);
        this
    }

    pub fn update(&mut self, starting_height: Height) {
        let keep = self.resolution.update(starting_height);
        self.update_dates(keep);
    }

    fn update_dates(&self, from: usize) {
        let to = self.first_height.len();
        match &self.date_source {
            DateSource::Period(date) => {
                self.dates
                    .update_at(from, (from..to).map(|i| date(I::from(i))));
            }
            DateSource::FirstTimestamp(timestamps) => {
                let heights: Vec<_> = self
                    .first_height
                    .collect_range_at(from, to)
                    .into_iter()
                    .map(Height::to_usize)
                    .collect();
                self.dates.update_at(
                    from,
                    timestamps
                        .read_sorted_at(&heights)
                        .into_iter()
                        .map(Date::from),
                );
            }
        }
    }
}
