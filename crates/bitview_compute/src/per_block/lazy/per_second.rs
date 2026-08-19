use bitview_traversable::Traversable;
use brk_types::{StoredF32, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{
    LazyPerBlock, LazyRollingSumFromHeight, LazyRollingSumsFromHeight, PerSecond, Windows,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyPerSecondWindows(
    /// Per-second average over the named trailing window: the window's total
    /// count divided by its fixed duration in seconds. The divisor remains the
    /// full duration before enough history exists. At time-period indexes, the
    /// value is taken from the period's final block.
    pub Windows<LazyPerBlock<StoredF32, StoredU64>>,
);

impl LazyPerSecondWindows {
    pub fn new(
        name: &str,
        version: Version,
        source: &LazyRollingSumsFromHeight<StoredU64>,
    ) -> Self {
        fn window<const SECONDS: u32>(
            name: &str,
            version: Version,
            source: &LazyRollingSumFromHeight<StoredU64>,
        ) -> LazyPerBlock<StoredF32, StoredU64> {
            LazyPerBlock::from_resolutions::<PerSecond<SECONDS>>(
                name,
                version,
                source.height.read_only_boxed_clone(),
                &source.resolutions,
            )
        }

        Self(Windows {
            _24h: window::<86_400>(&format!("{name}_24h"), version, &source._24h),
            _1w: window::<604_800>(&format!("{name}_1w"), version, &source._1w),
            _1m: window::<2_592_000>(&format!("{name}_1m"), version, &source._1m),
            _1y: window::<31_536_000>(&format!("{name}_1y"), version, &source._1y),
        })
    }
}
