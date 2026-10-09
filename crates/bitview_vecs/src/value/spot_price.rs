use bitview_transforms::CentsUnsignedToSats;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock, Price};

/// The block-level spot price: stored cents, lazy USD.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct SpotPrice<M: StorageMode = Rw>(pub Price<PerBlock<Cents, M>>);

impl SpotPrice {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(Price::import(db, name, version, indexes)?))
    }

    /// Integer sats one US dollar buys: 100,000,000 divided by the price in USD per BTC.
    pub fn sats_per_dollar(&self, name: &str, version: Version) -> LazyPerBlock<Sats, Cents> {
        LazyPerBlock::from_resolutions::<CentsUnsignedToSats>(name, version, &self.cents)
    }
}
