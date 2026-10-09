use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillionSigned32, Years};
use bitview_vecs::{LazyPerBlock, PerBlock, PercentPerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, UnaryTransform};

use super::{Vecs, Velocity};

/// Years of issuance at a rate: one over the rate; NaN for a non-positive rate.
struct RateToYears;

impl UnaryTransform<PartsPerMillionSigned32, Years> for RateToYears {
    #[inline]
    fn apply(rate: PartsPerMillionSigned32) -> Years {
        let rate = f64::from(rate);
        Years::from(if rate > 0.0 { 1.0 / rate } else { f64::NAN })
    }
}

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let inflation_rate = PercentPerBlock::import(
            db,
            "cointime_adjusted_inflation_rate",
            version + Version::new(3),
            mappings,
        )?;
        let stock_to_flow = LazyPerBlock::from_resolutions::<RateToYears>(
            "cointime_adjusted_stock_to_flow",
            version,
            &inflation_rate.fixed,
        );
        let version = version + Version::ONE;
        Ok(Vecs {
            inflation_rate,
            stock_to_flow,
            velocity: Velocity {
                btc: PerBlock::import(db, "cointime_adjusted_velocity_btc", version, mappings)?,
                usd: PerBlock::import(db, "cointime_adjusted_velocity_usd", version, mappings)?,
            },
        })
    }
}
