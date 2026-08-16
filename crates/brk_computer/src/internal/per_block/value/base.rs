use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats, SatsSigned, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode, UnaryTransform};

use crate::{
    indexes,
    internal::{
        CentsUnsignedToDollars, LazyPerBlock, NumericValue, PerBlock, SatsSignedToBitcoin,
        SatsToBitcoin,
    },
};

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

#[derive(Traversable)]
pub struct ValuePerBlock<M: StorageMode = Rw> {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: PerBlock<Sats, M>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: PerBlock<Cents, M>,
}

impl ValuePerBlock {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let sats = PerBlock::forced_import(db, &format!("{name}_sats"), version, indexes)?;

        let btc = LazyPerBlock::from_computed::<SatsToBitcoin>(
            name,
            version,
            sats.height.read_only_boxed_clone(),
            &sats,
        );

        let cents = PerBlock::forced_import(db, &format!("{name}_cents"), version, indexes)?;

        let usd = LazyPerBlock::from_computed::<CentsUnsignedToDollars>(
            &format!("{name}_usd"),
            version,
            cents.height.read_only_boxed_clone(),
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
