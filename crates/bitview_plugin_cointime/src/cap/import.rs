use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::{Convert, Quotient};
use bitview_vecs::{FiatPerBlock, LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, PerBlock};
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{Database, Ident, ReadableCloneableVec};

use super::{super::supply, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        subsidy_cents: &PerBlock<Cents>,
        supply: &supply::Vecs,
        realized_cap: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let thermo_cents =
            LazyPerBlock::from_resolutions::<Ident>("thermo_cap_cents", version, subsidy_cents);
        let thermo_usd =
            LazyPerBlock::from_lazy::<Convert, Cents>("thermo_cap", version, &thermo_cents);
        let investor = FiatPerBlock::import(db, "investor_cap", version, mappings)?;
        let investorness =
            LazyPercentPerBlock::from_ratio::<Cents, Cents, Quotient<PartsPerMillion32>>(
                "investorness",
                version,
                investor.cents.resolutions.height_source(),
                realized_cap,
                mappings,
            );
        let producerness =
            LazyPercentPerBlock::from_ratio::<Cents, Cents, Quotient<PartsPerMillion32>>(
                "producerness",
                version,
                &thermo_cents.height,
                realized_cap,
                mappings,
            );
        Ok(Vecs {
            thermo: LazyFiatPerBlock {
                usd: thermo_usd,
                cents: thermo_cents,
            },
            investor,
            active: LazyPerBlock::from_lazy::<Ident, Dollars>(
                "active_cap",
                version,
                &supply.active.usd,
            ),
            vaulted: LazyPerBlock::from_lazy::<Ident, Dollars>(
                "vaulted_cap",
                version,
                &supply.vaulted.usd,
            ),
            cointime: FiatPerBlock::import(db, "cointime_cap", version + Version::ONE, mappings)?,
            investorness,
            producerness,
        })
    }
}
