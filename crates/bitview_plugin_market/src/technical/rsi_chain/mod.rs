mod compute;

pub use compute::compute;

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, PartsPerMillionSigned64, StoredF32, Version};
use vecdb::{Database, Rw, StorageMode, UnaryTransform};

use bitview_compute::{LazyPerBlock, PerBlock, PercentPerBlock};

struct Gain;

impl UnaryTransform<StoredF32, StoredF32> for Gain {
    fn apply(value: StoredF32) -> StoredF32 {
        StoredF32::from((*value).max(0.0))
    }
}

struct Loss;

impl UnaryTransform<StoredF32, StoredF32> for Loss {
    fn apply(value: StoredF32) -> StoredF32 {
        StoredF32::from((-*value).max(0.0))
    }
}

#[derive(Traversable)]
pub struct RsiChain<M: StorageMode = Rw> {
    #[traversable(hidden)]
    gains: LazyPerBlock<StoredF32>,
    #[traversable(hidden)]
    losses: LazyPerBlock<StoredF32>,
    #[traversable(hidden)]
    average_gain: PerBlock<StoredF32, M>,
    #[traversable(hidden)]
    average_loss: PerBlock<StoredF32, M>,
    /// Wilder-smoothed average gain divided by the sum of Wilder-smoothed
    /// average gain and loss. Returns 50% when both averages are zero.
    pub rsi: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    rsi_min: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    rsi_max: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    stoch_rsi: PercentPerBlock<PartsPerMillion32, M>,
    /// Simple moving average of Stochastic RSI over three times the chain's
    /// base interval. Stochastic RSI locates RSI within its trailing RSI range.
    pub stoch_rsi_k: PercentPerBlock<PartsPerMillion32, M>,
    /// Simple moving average of the Stochastic RSI K line over three times the
    /// chain's base interval.
    pub stoch_rsi_d: PercentPerBlock<PartsPerMillion32, M>,
}

pub fn forced_import(
    db: &Database,
    tf: &str,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    returns: &LazyPerBlock<StoredF32, PartsPerMillionSigned64>,
) -> Result<RsiChain> {
    macro_rules! import {
        ($name:expr) => {
            PerBlock::forced_import(db, &format!("rsi_{}_{}", $name, tf), version, indexes)?
        };
    }

    macro_rules! percent_import {
        ($name:expr) => {
            PercentPerBlock::forced_import(db, &format!("rsi_{}_{}", $name, tf), version, indexes)?
        };
    }

    let average_gain = import!("average_gain");
    let average_loss = import!("average_loss");
    let rsi = PercentPerBlock::forced_import(db, &format!("rsi_{tf}"), version, indexes)?;

    Ok(RsiChain {
        gains: LazyPerBlock::from_lazy::<Gain, PartsPerMillionSigned64>(
            &format!("rsi_gains_{tf}"),
            version,
            returns,
        ),
        losses: LazyPerBlock::from_lazy::<Loss, PartsPerMillionSigned64>(
            &format!("rsi_losses_{tf}"),
            version,
            returns,
        ),
        average_gain,
        average_loss,
        rsi,
        rsi_min: percent_import!("min"),
        rsi_max: percent_import!("max"),
        stoch_rsi: percent_import!("stoch"),
        stoch_rsi_k: percent_import!("stoch_k"),
        stoch_rsi_d: percent_import!("stoch_d"),
    })
}
