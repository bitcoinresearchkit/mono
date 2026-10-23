use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::PercentPerBlock;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

/// A part of a cohort quantity and its share of the cohort's whole.
#[derive(Traversable)]
pub struct Part<T: Clone, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: T,
    /// Share of the cohort's total.
    pub share: PercentPerBlock<PartsPerMillion32, M>,
}
impl<T: Clone> Part<T> {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        v: Version,
        mappings: &Mappings,
        value: T,
    ) -> Result<Self> {
        Ok(Self {
            value,
            share: PercentPerBlock::import(db, &format!("{name}_share"), v, mappings)?,
        })
    }
}
