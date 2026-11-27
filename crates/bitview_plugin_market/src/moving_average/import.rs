use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{CentsFract, PriceRatio};
use bitview_vecs::{LazyPerBlock, PriceWithRatio, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Convert, Database, Ident, LazyVec, ReadableCloneableVec};

use super::{
    Vecs,
    periods::Periods,
    sma::{PiCycle, SmaSource, SmaVecs, StoredSma},
};

const EMA_VERSION: Version = Version::new(3);

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let sma_version = version + Version::TWO;
        let periods = Periods::try_from_fn(|suffix, days| -> Result<_> {
            let source = SmaSource::import(db, suffix, days, sma_version, mappings)?;
            Ok(StoredSma {
                price: PriceWithRatio::import(db, &format!("price_sma_{suffix}"), sma_version, &source.cents, mappings)?,
                source,
            })
        })?;
        let mayer_multiple = LazyPerBlock::from_lazy::<Ident, PriceRatio>(
            "mayer_multiple",
            sma_version,
            &periods._200d.price.relative.ratio,
        );
        let ema_version = version + EMA_VERSION;
        let ema_stored = Periods::try_from_fn(|suffix, _| {
            import_cached(db, &format!("price_ema_{suffix}_state"), ema_version, mappings)
        })?;
        let ema = ema_stored.try_map(|suffix, stored| {
            let name = format!("price_ema_{suffix}");
            // Whole cents for the price family; the stored state keeps the exact average.
            let cents = LazyVec::<Height, Cents, CentsFract>::transformed::<Convert>(
                &format!("{name}_cents"),
                ema_version,
                stored.read_only_boxed_clone(),
            );
            PriceWithRatio::import(db, &name, ema_version, &cents, mappings)
        })?;
        Ok(Vecs {
            sma: SmaVecs {
                periods,
                mayer_multiple,
            },
            pi_cycle: PiCycle::import(db, sma_version, mappings)?,
            ema,
            ema_stored,
        })
    }
}
