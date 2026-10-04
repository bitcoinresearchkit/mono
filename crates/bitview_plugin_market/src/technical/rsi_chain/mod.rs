use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{FixedRatioPerBlock, LazyPerBlock, PerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode, UnaryTransform};

mod compute;

struct Gain;

impl UnaryTransform<Ratio, Ratio> for Gain {
    fn apply(value: Ratio) -> Ratio {
        Ratio::new((*value).max(0.0))
    }
}

struct Loss;

impl UnaryTransform<Ratio, Ratio> for Loss {
    fn apply(value: Ratio) -> Ratio {
        Ratio::new((-*value).max(0.0))
    }
}

#[derive(Traversable)]
pub struct RsiChain<M: StorageMode = Rw> {
    #[traversable(hidden)]
    gains: LazyPerBlock<Ratio>,
    #[traversable(hidden)]
    losses: LazyPerBlock<Ratio>,
    #[traversable(hidden)]
    average_gain: PerBlock<Ratio, M>,
    #[traversable(hidden)]
    average_loss: PerBlock<Ratio, M>,
    /// Wilder-smoothed average gain divided by the sum of Wilder-smoothed
    /// average gain and loss. The result ranges from 0% to 100%; values above
    /// 50% mean smoothed gains exceed losses, and values below 50% mean losses
    /// exceed gains. Returns 50% when both averages are zero.
    pub rsi: FixedRatioPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    rsi_min: FixedRatioPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    rsi_max: FixedRatioPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    stoch_rsi: FixedRatioPerBlock<PartsPerMillion32, M>,
    /// Simple moving average of Stochastic RSI over three times the chain's
    /// base interval. Stochastic RSI locates RSI within its trailing RSI range,
    /// from 0% at the range minimum to 100% at the range maximum; this K line
    /// smooths that position.
    pub stoch_rsi_k: FixedRatioPerBlock<PartsPerMillion32, M>,
    /// Signal line for Stochastic RSI: the simple moving average of its K line
    /// over three times the chain's base interval. K above D means the smoothed
    /// position of RSI within its recent range is rising relative to this
    /// slower signal; K below D means it is falling.
    pub stoch_rsi_d: FixedRatioPerBlock<PartsPerMillion32, M>,
}

impl RsiChain {
    pub(crate) fn import(
        db: &Database,
        tf: &str,
        version: Version,
        mappings: &MappingsVecs,
        returns: &LazyPerBlock<Ratio, PartsPerMillionSigned64>,
    ) -> Result<Self> {
        macro_rules! import {
            ($name:expr) => {
                PerBlock::import(db, &format!("rsi_{}_{}", $name, tf), version, mappings)?
            };
        }

        macro_rules! percent_import {
            ($name:expr) => {
                FixedRatioPerBlock::import(db, &format!("rsi_{}_{}", $name, tf), version, mappings)?
            };
        }

        let average_gain = import!("average_gain");
        let average_loss = import!("average_loss");
        let rsi = FixedRatioPerBlock::import(db, &format!("rsi_{tf}"), version, mappings)?;

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
}
