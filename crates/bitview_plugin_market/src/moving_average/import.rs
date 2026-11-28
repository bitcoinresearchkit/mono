use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{CentsFract, PriceRatio};
use bitview_vecs::{LazyPerBlock, PriceWithRatio, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Convert, Database, Ident, LazyVec, ReadableCloneableVec};

use super::{
    Vecs,
    periods::{Periods, SmaPeriods},
    sma::{PiCycle, SmaVecs, StoredSma},
};

const SMA_VERSION: Version = Version::new(3);
const EMA_VERSION: Version = Version::new(3);

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let sma_version = version + SMA_VERSION;
        let periods = SmaPeriods::try_from_fn(|suffix, _| -> Result<_> {
            let name = format!("price_sma_{suffix}");
            let cents = import_cached(db, &format!("{name}_cents_source"), sma_version, mappings)?;
            Ok(StoredSma {
                price: PriceWithRatio::import(db, &name, sma_version, &cents, mappings)?,
                cents,
                window: Default::default(),
            })
        })?;
        let mayer_multiple = LazyPerBlock::from_lazy::<Ident, PriceRatio>(
            "mayer_multiple",
            sma_version,
            &periods._200d.price.relative.ratio,
        );
        let pi_cycle = PiCycle::new(sma_version, &periods, mappings);
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
            pi_cycle,
            ema,
            ema_stored,
        })
    }
}
