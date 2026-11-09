use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, FiatType, IndexSources, LazyFiatPerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, Rw, StorageMode, WritableVec};

use super::import_stored;

/// One cohort's fiat amount, stored in cents as `{name}_cents` and shown in every fiat unit.
#[derive(Traversable)]
pub struct Fiat<C: FiatType = Cents, M: StorageMode = Rw> {
    #[traversable(flatten)]
    value: LazyFiatPerBlock<C>,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, C, M>,
}

impl<C: FiatType> Fiat<C> {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let stored = import_stored(db, &format!("{name}_cents"), version)?;
        let value = LazyFiatPerBlock::from_cents_source(name, version, &stored, indexes);
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, value: C) {
        self.stored.push(value);
    }
}
