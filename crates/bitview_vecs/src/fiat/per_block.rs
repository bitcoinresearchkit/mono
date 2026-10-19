use bitview_compute::{NumericValue, Quantity};
use bitview_primitives::CentsFract;
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Dollars, Version};
use schemars::JsonSchema;
use vecdb::{Convert, Database, Rw};

use crate::{Fiat, IndexSources, LazyPerBlock, PerBlock};

/// A cents type that converts to dollars.
pub trait FiatType:
    NumericValue + JsonSchema + Quantity<Fract = CentsFract> + Into<Dollars>
{
}

impl FiatType for Cents {}

impl FiatType for CentsSigned {}

/// Stored cents with their lazy USD view.
pub type FiatPerBlock<C, M = Rw> = Fiat<PerBlock<C, M>, LazyPerBlock<Dollars, C>>;

impl<C: FiatType> FiatPerBlock<C> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_resolutions::<Convert>(name, version, &cents);
        Ok(Self { usd, cents })
    }
}
