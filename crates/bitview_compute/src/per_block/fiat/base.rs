use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, CentsSigned, Dollars, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode, UnaryTransform};

use crate::{CentsSignedToDollars, CentsUnsignedToDollars, LazyPerBlock, NumericValue, PerBlock};

/// Trait that associates a cents type with its transform to Dollars.
pub trait FiatType: NumericValue + JsonSchema {
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
#[derive(Traversable)]
pub struct FiatPerBlock<C: FiatType, M: StorageMode = Rw> {
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, C>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: PerBlock<C, M>,
}

impl<C: FiatType> FiatPerBlock<C> {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::forced_import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_computed::<C::ToDollars>(
            name,
            version,
            cents.height.read_only_boxed_clone(),
            &cents,
        );
        Ok(Self { usd, cents })
    }
}
