use bitview_primitives::Date;
use bitview_traversable::Traversable;
use bitview_vecs::RangeMapVec;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use rangeindex::RangeMapWriter;
use vecdb::{AnyVec, ReadableCloneableVec, Rw, StorageMode, VecIndex};

use super::ResolutionVecs;

/// Version of the date rule on top of the mapping's.
const DATE_VERSION: Version = Version::ONE;

/// Resident date and first-height lookups, updated together before publication.
#[derive(Deref, DerefMut, Traversable)]
pub struct DatedResolutionVecs<I: VecIndex, M: StorageMode = Rw> {
    /// UTC calendar date in `YYYY-MM-DD` format of the time period's start. At
    /// `day1`, this is the represented calendar day; at coarser indexes, the
    /// period's first day, whether or not a block falls on it.
    pub date: RangeMapVec<I, Date>,
    dates: M::WriteOnly<RangeMapWriter<Date, I>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    resolution: ResolutionVecs<I, M>,
    period_date: M::WriteOnly<fn(I) -> Date>,
}

impl<I: VecIndex> DatedResolutionVecs<I> {
    /// `period_date` maps a period to its start date.
    pub fn new(mapping: &impl ReadableCloneableVec<Height, I>, period_date: fn(I) -> Date) -> Self {
        let dates = RangeMapWriter::default();
        let this = Self {
            date: RangeMapVec::new("date", mapping.version() + DATE_VERSION, dates.shared()),
            dates,
            resolution: ResolutionVecs::new(mapping),
            period_date,
        };
        this.update_dates(0);
        this
    }

    pub fn update(&mut self, starting_height: Height) {
        if let Some(keep) = self.resolution.update(starting_height) {
            self.update_dates(keep);
        }
    }

    fn update_dates(&self, from: usize) {
        let to = self.first_height.len();
        self.dates
            .update_at(from, (from..to).map(|i| (self.period_date)(I::from(i))));
    }
}
