use bitview_compute::{NumericValue, Quantity};
use bitview_primitives::CentsFract;
use bitview_transforms::{CentsSignedToDollars, CentsUnsignedToDollars};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Dollars, Version};
use schemars::JsonSchema;
use vecdb::{Database, Rw, UnaryTransform};

use crate::{Fiat, IndexSources, LazyPerBlock, PerBlock};

/// Trait that associates a cents type with its transform to Dollars.
pub trait FiatType: NumericValue + JsonSchema + Quantity<Fract = CentsFract> {
    type ToDollars: UnaryTransform<Self, Dollars>;
}

impl FiatType for Cents {
    type ToDollars = CentsUnsignedToDollars;
}

impl FiatType for CentsSigned {
    type ToDollars = CentsSignedToDollars;
}

/// Height-indexed fiat monetary value: cents (eager, integer) + usd (lazy, float).
/// Generic over `C` to support both `Cents` (unsigned) and `CentsSigned` (signed).
pub type FiatPerBlock<C, M = Rw> = Fiat<PerBlock<C, M>, LazyPerBlock<Dollars, C>>;

impl<C: FiatType> FiatPerBlock<C> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_resolutions::<C::ToDollars>(name, version, &cents);
        Ok(Self { usd, cents })
    }
}
