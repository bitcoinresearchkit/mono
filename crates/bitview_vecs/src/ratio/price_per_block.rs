use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableCloneableVec, Rw};

use crate::{IndexSources, LazyRatioPerBlock, PerBlock, Price, PriceWithRatio};

pub type PriceWithRatioPerBlock<M = Rw> = PriceWithRatio<Price<PerBlock<Cents, M>>>;

impl PriceWithRatioPerBlock {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let price = Price::forced_import(db, name, version, indexes)?;
        let ratio = LazyRatioPerBlock::from_price_source(
            &format!("{name}_ratio"),
            version,
            price.cents.resolutions.height_source(),
            spot_price,
            indexes,
        );
        Ok(Self {
            price,
            relative: ratio,
        })
    }
}
