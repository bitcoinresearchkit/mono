use bitview_compute::NumericValue;
use bitview_transforms::Convert;
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Dollars, Sats, SatsSigned, Version};
use schemars::JsonSchema;
use vecdb::{Database, Rw};

use crate::{IndexSources, LazyPerBlock, PerBlock, Value};

/// A sats type that converts to bitcoin.
pub trait AmountType: NumericValue + JsonSchema + Into<Bitcoin> {}

impl AmountType for Sats {}

impl AmountType for SatsSigned {}

/// Sats and cents each own one shared source cache.
pub type ValuePerBlock<M = Rw> = Value<
    PerBlock<Sats, M>,
    PerBlock<Cents, M>,
    LazyPerBlock<Bitcoin, Sats>,
    LazyPerBlock<Dollars, Cents>,
>;

impl ValuePerBlock {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let sats = PerBlock::import(db, &format!("{name}_sats"), version, indexes)?;

        let btc = LazyPerBlock::from_resolutions::<Convert>(name, version, &sats);

        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;

        let usd =
            LazyPerBlock::from_resolutions::<Convert>(&format!("{name}_usd"), version, &cents);

        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }
}
