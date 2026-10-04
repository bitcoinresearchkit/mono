use bitview_transforms::{CentsUnsignedToSats, Convert};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Dollars, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock, Price};

/// Stored cents with integer sats per USD, preserving the spot-price conversion.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct SpotPrice<M: StorageMode = Rw>(
    pub Price<PerBlock<Cents, M>, LazyPerBlock<Dollars, Cents>, LazyPerBlock<Sats, Cents>>,
);

impl SpotPrice {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_resolutions::<Convert>(name, version, &cents);
        let sats = LazyPerBlock::from_resolutions::<CentsUnsignedToSats>(
            &format!("{name}_sats"),
            version,
            &cents,
        );
        Ok(Self(Price { usd, cents, sats }))
    }
}
