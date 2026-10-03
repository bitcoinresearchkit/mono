mod dated;

use bitview_traversable::Traversable;
use bitview_vecs::{RangeMapLookupVec, RangeMapVec};
use brk_types::Height;
use rangeindex::SharedRangeMap;
use vecdb::{AnyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, VecIndex, VecValue};

pub use self::dated::DatedResolutionVecs;

/// Resident period boundaries, incrementally maintained from the height mapping.
#[derive(Clone, Traversable)]
pub struct ResolutionVecs<I: VecIndex> {
    /// Lowest block height whose resolution index is at least the requested
    /// index. Empty indexes therefore use the first height of the next populated
    /// index; indexes preceding the first populated one resolve to height 0.
    pub first_height: RangeMapVec<I, Height>,
    #[traversable(skip)]
    boundaries: SharedRangeMap<Height, I>,
    #[traversable(skip)]
    mapping: ReadableBoxedVec<Height, I>,
    #[traversable(skip)]
    indexed_height: usize,
}

impl<I: VecIndex> ResolutionVecs<I> {
    pub fn new(mapping: &impl ReadableCloneableVec<Height, I>) -> Self {
        let boundaries = SharedRangeMap::new(Vec::new());
        let mut this = Self {
            first_height: RangeMapVec::new("first_height", mapping.version(), boundaries.clone()),
            boundaries,
            mapping: mapping.read_only_boxed_clone(),
            indexed_height: 0,
        };
        this.update(Height::ZERO);
        this
    }

    /// Published reverse view; the original mapping remains the update source.
    pub fn height_lookup(&self) -> RangeMapLookupVec<Height, I>
    where
        I: VecValue,
    {
        RangeMapLookupVec::new(&self.boundaries, &self.mapping)
    }

    /// Keep boundaries through the final surviving block, then replay the
    /// affected height suffix. Returns the number of preserved period entries.
    pub fn update(&mut self, starting_height: Height) -> usize {
        let source_len = self.mapping.len();
        let from = starting_height
            .to_usize()
            .min(self.indexed_height)
            .min(source_len);
        if from == source_len && from == self.indexed_height {
            return self.first_height.len();
        }

        let keep = from
            .checked_sub(1)
            .and_then(|height| self.mapping.collect_one_at(height))
            .map_or(0, |period| period.to_usize() + 1);
        let mut values = Vec::new();
        self.mapping
            .for_each_chunk_at(from, source_len, &mut |height, periods| {
                for (offset, period) in periods.iter().enumerate() {
                    let len = period.to_usize() + 1;
                    debug_assert!(len >= keep + values.len());
                    values.resize(len - keep, Height::from(height + offset));
                }
            });
        self.boundaries.update_at(keep, values);
        self.indexed_height = source_len;
        keep
    }
}
