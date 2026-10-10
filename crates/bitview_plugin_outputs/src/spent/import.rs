use brk_error::Result;
use brk_types::Version;
use vecdb::{
    BytesVec, Database, ImportOptions, ImportableVec, LazyVec, MutableVec, ReadableCloneableVec,
};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version) -> Result<Self> {
        // TWO: 5-byte `Index40` values.
        let txin_index = MutableVec::<BytesVec<_, _>>::import_with(
            ImportOptions::new(db, "txin_index", version + Version::TWO)
                .with_saved_stamped_changes(10),
        )?;
        Ok(Vecs {
            txin_index_view: LazyVec::init(
                "spending_txin_index",
                Version::ZERO,
                txin_index.read_only_boxed_clone(),
                |_, index| index.get(),
            ),
            txin_index,
        })
    }
}
