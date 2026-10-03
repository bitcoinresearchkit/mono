use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyIndexedVec, LazyPerBlock};
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, StoredF64, Version};
use vecdb::{Database, EagerVec, Ident, ImportableVec, ReadableBoxedVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v1 = version + Version::ONE;
        let hodl_bank = EagerVec::forced_import(db, "hodl_bank", v1)?;
        let value_source = LazyIndexedVec::new(
            "reserve_risk_source",
            v1,
            &hodl_bank,
            spot_price,
            |_, hodl_bank, spot| StoredF64::from(Dollars::from(spot)) / hodl_bank,
        );
        Ok(Vecs {
            vocdd_median_1y: EagerVec::forced_import(db, "vocdd_median_1y", v1)?,
            hodl_bank,
            value: LazyPerBlock::from_height_source::<Ident>(
                "reserve_risk",
                v1,
                &value_source,
                mappings,
            ),
        })
    }
}
