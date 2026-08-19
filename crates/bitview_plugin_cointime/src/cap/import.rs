use brk_error::Result;

use brk_types::{Cents, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;
use bitview_compute::{
    CentsUnsignedToDollars, FiatPerBlock, Identity, LazyFiatPerBlock, LazyPerBlock, PerBlock,
    RatioPerBlock,
};

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    subsidy_cents: &PerBlock<Cents>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, subsidy_cents)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        subsidy_cents: &PerBlock<Cents>,
    ) -> Result<Self> {
        let thermo_cents = LazyPerBlock::from_computed::<Identity<Cents>>(
            "thermo_cap_cents",
            version,
            subsidy_cents.height.read_only_boxed_clone(),
            subsidy_cents,
        );
        let thermo_usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            "thermo_cap",
            version,
            &thermo_cents,
        );
        Ok(Self {
            thermo: LazyFiatPerBlock {
                usd: thermo_usd,
                cents: thermo_cents,
            },
            investor: FiatPerBlock::forced_import(db, "investor_cap", version, indexes)?,
            vaulted: FiatPerBlock::forced_import(db, "vaulted_cap", version, indexes)?,
            active: FiatPerBlock::forced_import(db, "active_cap", version, indexes)?,
            cointime: FiatPerBlock::forced_import(
                db,
                "cointime_cap",
                version + Version::ONE,
                indexes,
            )?,
            aviv: RatioPerBlock::forced_import(db, "aviv", version, indexes)?,
        })
    }
}
