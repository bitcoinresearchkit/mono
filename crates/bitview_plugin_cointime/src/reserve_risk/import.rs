use brk_error::Result;

use brk_types::{Cents, Dollars, Height, StoredF64, Version};
use vecdb::{CachedBoxedVec, Database, EagerVec, ImportableVec, ReadableCloneableVec};

use super::Vecs;
use bitview_compute::{CACHE_BUDGET, Identity, LazyIndexedVec, LazyPerBlock};

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    spot_price: &CachedBoxedVec<Height, Cents>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, spot_price)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v1 = version + Version::ONE;
        let hodl_bank = EagerVec::forced_import(db, "hodl_bank", v1)?;
        let value_source = LazyIndexedVec::new(
            "reserve_risk_source",
            v1,
            hodl_bank.read_only_boxed_clone(),
            spot_price.clone(),
            |_, hodl_bank, spot| StoredF64::from(Dollars::from(spot)) / hodl_bank,
        );
        Ok(Self {
            vocdd_median_1y: EagerVec::forced_import(db, "vocdd_median_1y", v1)?,
            hodl_bank,
            value: LazyPerBlock::from_height_source::<Identity<StoredF64>>(
                "reserve_risk",
                v1,
                CACHE_BUDGET.wrap(value_source),
                indexes,
            ),
        })
    }
}
