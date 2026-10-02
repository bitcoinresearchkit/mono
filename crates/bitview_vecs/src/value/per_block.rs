use bitview_compute::NumericValue;
use bitview_transforms::{CentsUnsignedToDollars, SatsSignedToBitcoin, SatsToBitcoin};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Dollars, Sats, SatsSigned, Version};
use schemars::JsonSchema;
use vecdb::{Database, Rw, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, PerBlock, Value};

/// Trait that associates a sats type with its transform to Bitcoin.
pub trait AmountType: NumericValue + JsonSchema {
    type ToBitcoin: UnaryTransform<Self, Bitcoin>;
}

impl AmountType for Sats {
    type ToBitcoin = SatsToBitcoin;
}

impl AmountType for SatsSigned {
    type ToBitcoin = SatsSignedToBitcoin;
}

/// Sats and cents each own one shared source cache.
pub type ValuePerBlock<M = Rw> = Value<
    PerBlock<Sats, M>,
    PerBlock<Cents, M>,
    LazyPerBlock<Bitcoin, Sats>,
    LazyPerBlock<Dollars, Cents>,
>;

impl ValuePerBlock {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let sats = PerBlock::forced_import(db, &format!("{name}_sats"), version, indexes)?;

        let btc = LazyPerBlock::from_resolutions::<SatsToBitcoin>(name, version, &sats);

        let cents = PerBlock::forced_import(db, &format!("{name}_cents"), version, indexes)?;

        let usd = LazyPerBlock::from_resolutions::<CentsUnsignedToDollars>(
            &format!("{name}_usd"),
            version,
            &cents,
        );

        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }
}
