use brk_error::Result;

use brk_types::{PartsPerMillionSigned64, StoredF32, Version};
use vecdb::Database;

use super::{MacdChain, Vecs, rsi_chain};
use bitview_compute::{LazyPerBlock, PerBlock, RatioPerBlock, WindowsTo1m};

const VERSION: Version = Version::new(4);

fn forced_import_macd(
    db: &Database,
    tf: &str,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
) -> Result<MacdChain> {
    let line = PerBlock::forced_import(db, &format!("macd_line_{tf}"), version, mappings)?;
    let signal = PerBlock::forced_import(db, &format!("macd_signal_{tf}"), version, mappings)?;

    let histogram =
        PerBlock::forced_import(db, &format!("macd_histogram_{tf}"), version, mappings)?;

    Ok(MacdChain {
        ema_fast: PerBlock::forced_import(db, &format!("macd_ema_fast_{tf}"), version, mappings)?,
        ema_slow: PerBlock::forced_import(db, &format!("macd_ema_slow_{tf}"), version, mappings)?,
        line,
        signal,
        histogram,
    })
}

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    returns: &LazyPerBlock<StoredF32, PartsPerMillionSigned64>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, returns)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        returns: &LazyPerBlock<StoredF32, PartsPerMillionSigned64>,
    ) -> Result<Self> {
        let v = version + VERSION;

        let rsi = WindowsTo1m::try_from_fn(|tf| {
            rsi_chain::forced_import(db, tf, v + Version::TWO, mappings, returns)
        })?;
        let macd = WindowsTo1m::try_from_fn(|tf| forced_import_macd(db, tf, v, mappings))?;

        let pi_cycle = RatioPerBlock::forced_import_ppm(db, "pi_cycle", v, mappings)?;

        Ok(Self {
            rsi,
            pi_cycle,
            macd,
        })
    }
}
