use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CentsFract;
use bitview_transforms::Convert;
use bitview_vecs::{PriceWithRatio, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{Vecs, sma::SmaVecs, vecs::EmaPeriodId};

const EMA_VERSION: Version = Version::new(3);

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        blocks: &BlocksVecs,
    ) -> Result<Self> {
        let sma_prefix_sum = import_cached(db, "price_sma_prefix_sum", version + Version::ONE)?;
        let sma = SmaVecs::import(db, version, mappings, &blocks.lookback, &sma_prefix_sum)?;
        let ema_version = version + EMA_VERSION;
        let ema_stored = EmaPeriodId::try_series(|period| {
            import_cached(
                db,
                &format!("price_ema_{}_state", period.suffix()),
                ema_version,
            )
        })?;
        let ema = EmaPeriodId::try_series(|period| {
            let name = format!("price_ema_{}", period.suffix());
            // Whole cents for the price family; the stored state keeps the exact average.
            let cents = LazyVec::<Height, Cents, Height, CentsFract>::transformed::<Convert>(
                &format!("{name}_cents"),
                ema_version,
                period.select(&ema_stored).read_only_boxed_clone(),
            );
            PriceWithRatio::import(db, &name, ema_version, &cents, mappings)
        })?;
        Ok(Vecs {
            sma,
            ema,
            ema_stored,
            sma_prefix_sum,
        })
    }
}
