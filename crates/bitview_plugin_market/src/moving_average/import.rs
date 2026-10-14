use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CentsFract;
use bitview_vecs::{PriceWithRatio, import_cached, ops::Convert};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{Vecs, ema_vecs::EmaVecs, sma::SmaVecs};

const EMA_VERSION: Version = Version::new(3);

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let sma_prefix_sum = import_cached(db, "price_sma_prefix_sum", version + Version::ONE)?;
        let sma = SmaVecs::import(db, version, mappings, &sma_prefix_sum)?;
        let ema_version = version + EMA_VERSION;
        let ema_stored = EmaVecs::try_from_fn(|suffix, _| {
            import_cached(db, &format!("price_ema_{suffix}_state"), ema_version)
        })?;
        let ema = ema_stored.try_map(|suffix, stored| {
            let name = format!("price_ema_{suffix}");
            // Whole cents for the price family; the stored state keeps the exact average.
            let cents = LazyVec::<Height, Cents, Height, CentsFract>::transformed::<Convert>(
                &format!("{name}_cents"),
                ema_version,
                stored.read_only_boxed_clone(),
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
