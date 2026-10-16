use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PriceRatio, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PriceWithRatio};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, Ident, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::priced::PricedSmaVec;

#[derive(Traversable)]
pub struct SmaVecs<M: StorageMode = Rw> {
    /// Uses a trailing 7-day monotonic-time window.
    pub _1w: PriceWithRatio<M>,
    /// Uses a trailing 8-day monotonic-time window.
    pub _8d: PriceWithRatio<M>,
    /// Uses a trailing 13-day monotonic-time window.
    pub _13d: PriceWithRatio<M>,
    /// Uses a trailing 21-day monotonic-time window.
    pub _21d: PriceWithRatio<M>,
    /// Uses a trailing 30-day monotonic-time window.
    pub _1m: PriceWithRatio<M>,
    /// Uses a trailing 34-day monotonic-time window.
    pub _34d: PriceWithRatio<M>,
    /// Uses a trailing 50-day monotonic-time window.
    pub _50d: PriceWithRatio<M>,
    /// Uses a trailing 55-day monotonic-time window.
    pub _55d: PriceWithRatio<M>,
    /// Uses a trailing 89-day monotonic-time window.
    pub _89d: PriceWithRatio<M>,
    /// Uses a trailing 144-day monotonic-time window.
    pub _144d: PriceWithRatio<M>,
    /// Uses a trailing 200-day monotonic-time window.
    pub _200d: PriceWithRatio<M>,
    /// Uses a trailing 365-day monotonic-time window.
    pub _1y: PriceWithRatio<M>,
    /// Uses a trailing 730-day monotonic-time window.
    pub _2y: PriceWithRatio<M>,
    /// Uses a trailing 1,400-day monotonic-time window.
    pub _200w: PriceWithRatio<M>,
    /// Uses a trailing 1,460-day monotonic-time window.
    pub _4y: PriceWithRatio<M>,
    /// Mayer Multiple: Bitcoin spot price divided by its 200-day simple moving
    /// average, the name analysts use for that ratio, computed from cent-precision prices:
    /// null before the first exchange price (block 74,641), approximate while prices are a few
    /// cents.
    #[traversable(wrap = "200d", rename = "mayer_multiple")]
    pub mayer_multiple: LazyPerBlock<Ratio, Ratio>,
}

const VERSION: Version = Version::TWO;

impl SmaVecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        prefix_sum: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + VERSION;

        macro_rules! sma {
            ($name:literal, $days:expr) => {
                PriceWithRatio::import(
                    db,
                    concat!("price_sma_", $name),
                    version,
                    &PricedSmaVec::new(
                        concat!("price_sma_", $name, "_cents_source"),
                        version,
                        prefix_sum.read_only_boxed_clone(),
                        mappings.lookback.start_vec($days).read_only_boxed_clone(),
                    ),
                    mappings,
                )?
            };
        }

        let _200d = sma!("200d", 200);
        let mayer_multiple = LazyPerBlock::from_lazy::<Ident, PriceRatio>(
            "mayer_multiple",
            version,
            &_200d.relative.ratio,
        );

        Ok(Self {
            _1w: sma!("1w", 7),
            _8d: sma!("8d", 8),
            _13d: sma!("13d", 13),
            _21d: sma!("21d", 21),
            _1m: sma!("1m", 30),
            _34d: sma!("34d", 34),
            _50d: sma!("50d", 50),
            _55d: sma!("55d", 55),
            _89d: sma!("89d", 89),
            _144d: sma!("144d", 144),
            _200d,
            _1y: sma!("1y", 365),
            _2y: sma!("2y", 2 * 365),
            _200w: sma!("200w", 200 * 7),
            _4y: sma!("4y", 4 * 365),
            mayer_multiple,
        })
    }

    /// Spot divided by each average, stored.
    pub(crate) fn compute_ratios(
        &mut self,
        from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        for average in [
            &mut self._1w,
            &mut self._8d,
            &mut self._13d,
            &mut self._21d,
            &mut self._1m,
            &mut self._34d,
            &mut self._50d,
            &mut self._55d,
            &mut self._89d,
            &mut self._144d,
            &mut self._200d,
            &mut self._1y,
            &mut self._2y,
            &mut self._200w,
            &mut self._4y,
        ] {
            average.compute_ratio(from, spot, exit)?;
        }
        Ok(())
    }
}
