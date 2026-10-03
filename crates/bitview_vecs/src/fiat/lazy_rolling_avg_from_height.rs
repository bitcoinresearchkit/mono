use bitview_primitives::StoredF32;
use bitview_transforms::AvgCentsToUsd;
use brk_types::{Dollars, Height, Version};
use vecdb::ReadableCloneableVec;

use crate::{Fiat, FiatType, IndexSources, LazyPerBlock, LazyRollingAvgFromHeight};

pub type LazyRollingAvgFiatFromHeight<C> =
    Fiat<LazyRollingAvgFromHeight<C>, LazyPerBlock<Dollars, StoredF32>>;

impl<C: FiatType> LazyRollingAvgFiatFromHeight<C> {
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, C>,
        window_start: &impl ReadableCloneableVec<Height, Height>,
        indexes: &IndexSources,
    ) -> Self {
        let cents = LazyRollingAvgFromHeight::from_source(
            &format!("{name}_cents"),
            version,
            cumulative,
            window_start,
            indexes,
        );
        let usd =
            LazyPerBlock::from_resolutions::<AvgCentsToUsd>(name, version, &cents.resolutions);

        Self { usd, cents }
    }
}
