use bitview_compute::{NumericValue, compute_cumulative_sum_from_indexes};
use bitview_transforms::{CentsUnsignedToDollars, SatsSignedToBitcoin, SatsToBitcoin};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, SatsSigned, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableVec, Rw, UnaryTransform, VecIndex, VecValue};

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
    pub fn forced_import(
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

impl ValuePerBlock {
    #[allow(clippy::too_many_arguments)]
    pub fn compute_sats_from_indexes<A, B>(
        &mut self,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        compute_cumulative_sum_from_indexes(
            &mut self.sats.height,
            max_from,
            first_indexes,
            indexes_count,
            source,
            |value| value,
            exit,
        )
    }
}
