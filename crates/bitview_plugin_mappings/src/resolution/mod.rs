mod dated;

use bitview_traversable::Traversable;
use bitview_vecs::RangeMapVec;

use crate::RangeMapLookupVec;
use brk_types::{Height, Version};
use rangeindex::RangeMapWriter;
use vecdb::{
    AnyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecIndex,
};

pub use self::dated::DatedResolutionVecs;

/// Resident period boundaries, incrementally maintained from the height mapping. Read-only
/// clones keep the published views only: the updater stays with the computing plugin.
#[derive(Traversable)]
pub struct ResolutionVecs<I: VecIndex, M: StorageMode = Rw> {
    /// Lowest block height whose resolution index is at least the requested
    /// index. Empty indexes therefore use the first height of the next populated
    /// index; indexes preceding the first populated one resolve to height 0.
    pub first_height: RangeMapVec<I, Height>,
    boundaries: M::WriteOnly<RangeMapWriter<Height, I>>,
    #[traversable(skip)]
    mapping: ReadableBoxedVec<Height, I>,
    indexed_height: M::WriteOnly<usize>,
}

impl<I: VecIndex> ResolutionVecs<I> {
    pub fn new(mapping: &impl ReadableCloneableVec<Height, I>) -> Self {
        let mapping = mapping.read_only_boxed_clone();
        let boundaries = RangeMapWriter::default();
        let mut this = Self {
            first_height: RangeMapVec::new(
                "first_height",
                Version::ZERO,
                mapping.clone(),
                boundaries.shared(),
            ),
            boundaries,
            mapping,
            indexed_height: 0,
        };
        this.update(Height::ZERO);
        this
    }

    /// Published reverse view; the original mapping remains the update source.
    pub fn height_lookup(&self) -> RangeMapLookupVec<Height, I> {
        RangeMapLookupVec::new(self.boundaries.shared(), &self.mapping)
    }

    /// Keep boundaries through the final surviving block, then replay the
    /// affected height suffix. Returns the number of preserved period entries
    /// when the boundaries changed, `None` when no period started or ended.
    pub fn update(&mut self, starting_height: Height) -> Option<usize> {
        let source_len = self.mapping.len();
        let from = starting_height
            .to_usize()
            .min(self.indexed_height)
            .min(source_len);
        if from == source_len && from == self.indexed_height {
            return None;
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
        self.indexed_height = source_len;
        if values.is_empty() && keep == self.first_height.len() {
            return None;
        }
        self.boundaries.update_at(keep, values);
        Some(keep)
    }
}
