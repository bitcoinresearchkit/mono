use bitview_traversable::Compact;
use brk_error::Result;
use brk_types::Version;
use vecdb::{BytesVec, Database, ImportOptions, ImportableVec, MutableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version) -> Result<Self> {
        // TWO: 5-byte `Index40` values.
        let txin_index = MutableVec::<BytesVec<_, _>>::import_with(
            ImportOptions::new(db, "txin_index", version + Version::TWO)
                .with_saved_stamped_changes(10),
        )?;
        Ok(Vecs {
            txin_index: Compact::named(txin_index, "spending_txin_index"),
        })
    }
}
