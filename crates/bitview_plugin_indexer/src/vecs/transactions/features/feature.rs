use bitview_primitives::{Boolean, Count, Count16};
use bitview_traversable::{Compact, Traversable};
use brk_error::Result;
use brk_types::{Height, TxIndex, Version};
use vecdb::{
    AnyVec, Database, Ident, ImportableVec, LazyVec, PcoVec, ReadableCloneableVec, Rw, StorageMode,
};

/// A per-block transaction count, stored in 2 bytes per block and published as `Count`.
pub type BlockCount<M = Rw> = Compact<PcoVec<Height, Count16>, Count, M>;

/// One transaction feature: its per-transaction flag beside its per-block count.
#[derive(Traversable)]
pub struct FeatureVecs<M: StorageMode = Rw> {
    /// Whether the transaction is one of them.
    pub flag: M::Stored<PcoVec<TxIndex, Boolean>>,
    /// Value for the represented block.
    pub block: BlockCount<M>,
}

/// A transaction flag read as is, for plugins that show it beside their own series.
pub type FlagView = LazyVec<TxIndex, Boolean, TxIndex, Boolean>;

impl FeatureVecs {
    /// The flag as a view, for plugins that show it beside their own series.
    pub fn flag_view(&self) -> FlagView {
        LazyVec::transformed::<Ident>(
            self.flag.name(),
            Version::ZERO,
            self.flag.read_only_boxed_clone(),
        )
    }

    pub(crate) fn import(db: &Database, flag: &str, block: &str, version: Version) -> Result<Self> {
        Ok(Self {
            flag: PcoVec::import(db, flag, version)?,
            block: block_count(db, block, version)?,
        })
    }
}

/// A per-block transaction count without a per-transaction flag.
#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Value for the represented block.
    pub block: BlockCount<M>,
}

impl CountVecs {
    pub(crate) fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            block: block_count(db, name, version)?,
        })
    }
}

pub(crate) fn block_count(db: &Database, name: &str, version: Version) -> Result<BlockCount> {
    Ok(Compact::new(PcoVec::import(db, name, version)?))
}
