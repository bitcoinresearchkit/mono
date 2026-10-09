use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Float64;
use bitview_vecs::{LazyIndexedVec, LazyPerBlock};
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{Database, EagerVec, Ident, ImportableVec, ReadableBoxedVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v1 = version + Version::TWO;
        let hodl_bank = EagerVec::import(db, "hodl_bank", v1 + Version::ONE)?;
        let value_source = LazyIndexedVec::new(
            "reserve_risk_source",
            v1,
            &hodl_bank,
            spot_price,
            |_, hodl_bank: Float64, spot| Float64::new(f64::from(Dollars::from(spot)) / *hodl_bank),
        );
        Ok(Vecs {
            vocdd_median_1m: EagerVec::import(db, "vocdd_median_1m", v1)?,
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
