use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::CentsUnsignedToDollars;
use bitview_vecs::{FiatPerBlock, LazyFiatPerBlock, LazyPerBlock, PerBlock, RatioPerBlock};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Ident};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        subsidy_cents: &PerBlock<Cents>,
    ) -> Result<Self> {
        let thermo_cents =
            LazyPerBlock::from_resolutions::<Ident>("thermo_cap_cents", version, subsidy_cents);
        let thermo_usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            "thermo_cap",
            version,
            &thermo_cents,
        );
        Ok(Vecs {
            thermo: LazyFiatPerBlock {
                usd: thermo_usd,
                cents: thermo_cents,
            },
            investor: FiatPerBlock::import(db, "investor_cap", version, mappings)?,
            vaulted: FiatPerBlock::import(db, "vaulted_cap", version, mappings)?,
            active: FiatPerBlock::import(db, "active_cap", version, mappings)?,
            cointime: FiatPerBlock::import(db, "cointime_cap", version + Version::ONE, mappings)?,
            aviv: RatioPerBlock::import(db, "aviv", version, mappings)?,
        })
    }
}
