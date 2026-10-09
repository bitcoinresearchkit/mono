use bitview_collections::WindowsTo1m;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillionSigned64, Percent};
use bitview_vecs::{LazyPerBlock, PerBlock, RatioPerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{MacdChain, Vecs, rsi_chain};

const VERSION: Version = Version::new(5);

fn import_macd(
    db: &Database,
    tf: &str,
    version: Version,
    mappings: &MappingsVecs,
) -> Result<MacdChain> {
    let line = PerBlock::import(db, &format!("macd_line_{tf}"), version, mappings)?;
    let signal = PerBlock::import(db, &format!("macd_signal_{tf}"), version, mappings)?;

    let histogram = PerBlock::import(db, &format!("macd_histogram_{tf}"), version, mappings)?;

    Ok(MacdChain {
        ema_fast: PerBlock::import(db, &format!("macd_ema_fast_{tf}"), version, mappings)?,
        ema_slow: PerBlock::import(db, &format!("macd_ema_slow_{tf}"), version, mappings)?,
        line,
        signal,
        histogram,
    })
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        returns: &LazyPerBlock<Percent, PartsPerMillionSigned64>,
    ) -> Result<Self> {
        let v = version + VERSION;

        let rsi = WindowsTo1m::try_from_fn(|tf| {
            rsi_chain::RsiChain::import(db, tf, v + Version::new(3), mappings, returns)
        })?;
        let macd = WindowsTo1m::try_from_fn(|tf| import_macd(db, tf, v, mappings))?;

        let pi_cycle = RatioPerBlock::import(db, "pi_cycle", v, mappings)?;

        Ok(Vecs {
            rsi,
            pi_cycle,
            macd,
        })
    }
}
