use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Float64;
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlock, LazyWindowStartVec, PerBlock, PricedPerBlockCumulativeRolling,
    ops::Narrow, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{Database, ImportableVec, PcoVec, ReadableBoxedVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let reserve_risk_version = version + Version::TWO;
        let hodl_bank_sum = PerBlock::import(
            db,
            "hodl_bank_sum",
            reserve_risk_version + Version::ONE,
            mappings,
        )?;
        let value_source = LazyIndexedVec::new(
            "reserve_risk_source",
            reserve_risk_version,
            &hodl_bank_sum.height,
            spot_price,
            |_, hodl_bank: Float64, spot| Float64::new(f64::from(Dollars::from(spot)) / *hodl_bank),
        );
        Ok(Vecs {
            value: LazyPerBlock::from_height_source::<Narrow>(
                "reserve_risk",
                reserve_risk_version,
                &value_source,
                mappings,
            ),
            hodl_bank: LazyPerBlock::from_resolutions::<Narrow>(
                "hodl_bank",
                reserve_risk_version + Version::ONE,
                &hodl_bank_sum,
            ),
            hodl_bank_sum,
            vocdd: PricedPerBlockCumulativeRolling::import(
                db,
                "vocdd",
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
            vocdd_median_1m: PcoVec::import(db, "vocdd_median_1m", reserve_risk_version)?,
            vocdd_median_1m_window: Default::default(),
        })
    }
}
