mod boundary;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    BLOCKS_PER_DIFF_EPOCHS, BLOCKS_PER_HALVING, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4,
    Hour12, Minute10, Minute30, Month1, Month3, Month6, Timestamp, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyVec, CachedVec, Database, EagerVec, Exit, ImportableVec, LazyVec, PcoVec, ReadableBoxedVec,
    ReadableVec, Rw, StorageMode, Version,
};

use crate::internal::PerResolution;

pub use boundary::BoundaryTimestampVec;

/// Timestamps: monotonic height→timestamp + per-period timestamp lookups.
///
/// Time-based periods (minute10–year10) are lazy: `idx.to_timestamp()` is a pure
/// function of the index, so no storage or decompression is needed.
/// Block-based periods (halving, difficulty) are storage-free views of the raw
/// timestamp at each period's first block.
#[derive(Deref, DerefMut, Traversable)]
pub struct Timestamps<M: StorageMode = Rw> {
    /// Nondecreasing Unix timestamp in seconds at each block height, computed as
    /// the maximum of the current raw block-header timestamp and the preceding
    /// monotonic timestamp.
    pub monotonic: CachedVec<M::Stored<EagerVec<PcoVec<Height, Timestamp>>>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: PerResolution<
        LazyVec<Minute10, Timestamp, Minute10, Height>,
        LazyVec<Minute30, Timestamp, Minute30, Height>,
        LazyVec<Hour1, Timestamp, Hour1, Height>,
        LazyVec<Hour4, Timestamp, Hour4, Height>,
        LazyVec<Hour12, Timestamp, Hour12, Height>,
        LazyVec<Day1, Timestamp, Day1, Height>,
        LazyVec<Day3, Timestamp, Day3, Height>,
        LazyVec<Week1, Timestamp, Week1, Height>,
        LazyVec<Month1, Timestamp, Month1, Height>,
        LazyVec<Month3, Timestamp, Month3, Height>,
        LazyVec<Month6, Timestamp, Month6, Height>,
        LazyVec<Year1, Timestamp, Year1, Height>,
        LazyVec<Year10, Timestamp, Year10, Height>,
        BoundaryTimestampVec<Halving>,
        BoundaryTimestampVec<Epoch>,
    >,
}

impl Timestamps {
    pub(crate) fn forced_import_monotonic(
        db: &Database,
        version: Version,
    ) -> Result<CachedVec<EagerVec<PcoVec<Height, Timestamp>>>> {
        Ok(CachedVec::wrap(EagerVec::forced_import(
            db,
            "timestamp_monotonic",
            version,
        )?))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_locals(
        version: Version,
        monotonic: CachedVec<EagerVec<PcoVec<Height, Timestamp>>>,
        raw_timestamps: ReadableBoxedVec<Height, Timestamp>,
        minute10: &super::ResolutionVecs<Minute10>,
        minute30: &super::ResolutionVecs<Minute30>,
        hour1: &super::ResolutionVecs<Hour1>,
        hour4: &super::ResolutionVecs<Hour4>,
        hour12: &super::ResolutionVecs<Hour12>,
        day1: &super::DatedResolutionVecs<Day1>,
        day3: &super::DatedResolutionVecs<Day3>,
        week1: &super::DatedResolutionVecs<Week1>,
        month1: &super::DatedResolutionVecs<Month1>,
        month3: &super::DatedResolutionVecs<Month3>,
        month6: &super::DatedResolutionVecs<Month6>,
        year1: &super::DatedResolutionVecs<Year1>,
        year10: &super::DatedResolutionVecs<Year10>,
    ) -> Self {
        macro_rules! period {
            ($field:ident) => {
                LazyVec::init(
                    "timestamp",
                    version,
                    $field.first_height.read_only_boxed_clone(),
                    |idx, _: Height| idx.to_timestamp(),
                )
            };
        }

        Self {
            monotonic,
            resolutions: PerResolution {
                minute10: period!(minute10),
                minute30: period!(minute30),
                hour1: period!(hour1),
                hour4: period!(hour4),
                hour12: period!(hour12),
                day1: period!(day1),
                day3: period!(day3),
                week1: period!(week1),
                month1: period!(month1),
                month3: period!(month3),
                month6: period!(month6),
                year1: period!(year1),
                year10: period!(year10),
                halving: BoundaryTimestampVec::new(
                    raw_timestamps.clone(),
                    BLOCKS_PER_HALVING as usize,
                ),
                epoch: BoundaryTimestampVec::new(raw_timestamps, BLOCKS_PER_DIFF_EPOCHS as usize),
            },
        }
    }

    pub(crate) fn compute_monotonic(
        &mut self,
        indexer: &brk_indexer::Indexer,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<bool> {
        let rewrites_existing = usize::from(starting_height) < self.monotonic.len();
        let mut prev = None;
        self.monotonic.inner.compute_transform(
            starting_height,
            &indexer.vecs().blocks.timestamp,
            |(h, timestamp, this)| {
                if prev.is_none()
                    && let Some(prev_h) = h.decremented()
                {
                    prev.replace(this.collect_one(prev_h).unwrap());
                }
                let monotonic = prev.map_or(timestamp, |p| p.max(timestamp));
                prev.replace(monotonic);
                (h, monotonic)
            },
            exit,
        )?;
        if rewrites_existing {
            self.monotonic.invalidate();
        }
        Ok(rewrites_existing)
    }
}
